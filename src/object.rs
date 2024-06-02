use std::{any::{Any, TypeId}, collections::HashMap, ffi::CString, marker::PhantomData, mem, ptr};
use libc::c_char;

use crate::{lua_State, lua_call, lua_getfield, lua_insert, lua_pushvalue, lua_rotate, lua_type, push_lightuserdata, sys, Lua, LuaPush, LuaRead, LuaTable};

// use std::sync::OnceLock;
// fn hashmap() -> &'static HashMap<&'static str, Box<dyn LuaPush + 'static + Send>> {
//     static HASHMAP: OnceLock<HashMap<&'static str, Box<dyn LuaPush + 'static + Send>>> = OnceLock::new();
//     HASHMAP.get_or_init(|| {
//         let mut m = HashMap::new();
//         m
//     })
// }

// Called when an object inside Lua is being dropped.
#[inline]
extern "C" fn destructor_wrapper<T>(lua: *mut sys::lua_State) -> libc::c_int {
    unsafe {
        let obj = sys::lua_touserdata(lua, -1);
        ptr::drop_in_place(obj as *mut T);
        0
    }
}

extern "C" fn constructor_wrapper<T>(lua: *mut sys::lua_State) -> libc::c_int
where
    T: Default + Any,
{
    let t = T::default();
    let lua_data_raw =
        unsafe { sys::lua_newuserdata(lua, mem::size_of::<T>() as libc::size_t) };
    unsafe {
        ptr::write(lua_data_raw as *mut _, t);
    }
    let typeid = CString::new(format!("{:?}", TypeId::of::<T>())).unwrap();
    unsafe {
        sys::lua_getglobal(lua, typeid.as_ptr());
        sys::lua_setmetatable(lua, -2);
    }
    1
}

extern "C" fn index_metatable<'a, T>(lua: *mut sys::lua_State) -> libc::c_int
where
    T: Default + Any,
    &'a mut T: LuaRead,
{
    println!("aaaaaaaaaaaaaaaaaaakkkkkkkkkkkkkkkkkkkkkkkk");
    if let Some(key) = String::lua_read_with_pop(lua, 2, 0) {
        let typeid = CString::new(format!("{:?}_new", TypeId::of::<T>())).unwrap();
        unsafe {
            sys::lua_getglobal(lua, typeid.as_ptr());
            let t = lua_getfield(lua, -1, key.as_ptr() as *const i8);
            if t != sys::LUA_TFUNCTION {
                return 0;
            }
            lua_pushvalue(lua, 1);
            lua_pushvalue(lua, 2);
            lua_call(lua, 2, 1);
            1
        }
    } else {
        0
    }
    // let key: Option<String> = LuaRead::lua_read_with_pop(lua, 2, 0);


    // let val: Option<String> = LuaRead::lua_read_with_pop(lua, 0, 0);
    // println!("val = {:?}", key);
    // let key = 
    // let t = T::default();
    // let lua_data_raw =
    //     unsafe { sys::lua_newuserdata(lua, mem::size_of::<T>() as libc::size_t) };
    // unsafe {
    //     ptr::write(lua_data_raw as *mut _, t);
    // }
    // let typeid = CString::new(format!("{:?}", TypeId::of::<T>())).unwrap();
    // unsafe {
    //     sys::lua_getglobal(lua, typeid.as_ptr());
    //     sys::lua_setmetatable(lua, -2);
    // }
    // key.unwrap_or("xxx".to_string()).push_to_lua(lua);
    // 1
}

// constructor direct create light object,
// in rust we alloc the memory, avoid copy the memory
// in lua we get the object, we must free the memory
extern "C" fn constructor_light_wrapper<T>(lua: *mut sys::lua_State) -> libc::c_int
where
    T: Default + Any,
{
    let t = Box::into_raw(Box::new(T::default()));
    push_lightuserdata(unsafe { &mut *t }, lua, |_| {});

    let typeid = CString::new(format!("{:?}", TypeId::of::<T>())).unwrap();
    unsafe {
        sys::lua_getglobal(lua, typeid.as_ptr());
        sys::lua_setmetatable(lua, -2);
    }
    1
}

pub struct LuaObject<'a, T>
where T: Default + Any,
    &'a mut T: LuaRead {
    lua: *mut lua_State,
    light: bool,
    name: &'static str,
    marker: PhantomData<&'a T>,
}

impl<'a, T> LuaObject<'a, T>
where
    T: Default + Any,
    &'a mut T: LuaRead 
{
    pub fn new(lua: *mut lua_State, name: &'static str) -> LuaObject<'a, T> {
        LuaObject {
            lua,
            light: false,
            name,
            marker: PhantomData,
        }
    }

    pub fn new_light(lua: *mut lua_State, name: &'static str) -> LuaObject<'a, T> {
        LuaObject {
            lua,
            light: true,
            name,
            marker: PhantomData,
        }
    }

    pub fn ensure_matetable(&mut self) -> bool {
        let typeid = format!("{:?}", TypeId::of::<T>());
        let mut lua = Lua::from_existing_state(self.lua, false);
        match lua.query::<LuaTable, _>(typeid.clone()) {
            Some(_) => {
                true
            }
            None => unsafe {
                sys::lua_newtable(self.lua);

                let typeid = format!("{:?}", TypeId::of::<T>());
                // index "__name" corresponds to the hash of the TypeId of T
                "__typeid".push_to_lua(self.lua);
                (&typeid).push_to_lua(self.lua);
                sys::lua_settable(self.lua, -3);

                // index "__gc" call the object's destructor
                if !self.light {
                    "__gc".push_to_lua(self.lua);

                    sys::lua_pushcfunction(self.lua, destructor_wrapper::<T>);

                    sys::lua_settable(self.lua, -3);
                }

                "__index".push_to_lua(self.lua);
                sys::lua_pushcfunction(self.lua, index_metatable::<T>);
                // sys::lua_newtable(self.lua);
                sys::lua_rawset(self.lua, -3);

                "__newindex".push_to_lua(self.lua);
                sys::lua_newtable(self.lua);
                sys::lua_rawset(self.lua, -3);

                sys::lua_setglobal(self.lua, typeid.as_ptr() as *const c_char);
                
                let typeid = format!("{:?}_new", TypeId::of::<T>());
                sys::lua_newtable(self.lua);
                sys::lua_setglobal(self.lua, typeid.as_ptr() as *const c_char);
                false
            },
        }
    }

    pub fn create(&mut self) -> &mut LuaObject<'a, T> {
        self.ensure_matetable();
        unsafe {

            let name = CString::new(self.name).unwrap();
            if self.light {
                sys::lua_pushcfunction(self.lua, constructor_light_wrapper::<T>);
            } else {
                sys::lua_pushcfunction(self.lua, constructor_wrapper::<T>);
            }
            sys::lua_setglobal(self.lua, name.as_ptr());

            // sys::lua_getglobal(self.lua, name.as_ptr());
            // if !sys::lua_istable(self.lua, -1) {
            //     sys::lua_newtable(self.lua);
            //     sys::lua_setglobal(self.lua, name.as_ptr());
            //     sys::lua_getglobal(self.lua, name.as_ptr());
            // }
            // if sys::lua_istable(self.lua, -1) {
            //     sys::lua_newtable(self.lua);
            //     "__call".push_to_lua(self.lua);

            //     if self.light {
            //         sys::lua_pushcfunction(self.lua, constructor_light_wrapper::<T>);
            //         sys::lua_settable(self.lua, -3);
            //     } else {
            //         sys::lua_pushcfunction(self.lua, constructor_wrapper::<T>);
            //         sys::lua_settable(self.lua, -3);
            //     }

            //     sys::lua_setmetatable(self.lua, -2);
            // }
            // sys::lua_pop(self.lua, 1);
        }
        self
    }

    pub fn add_method_get<P>(&mut self, name: &str, param: P) -> &mut LuaObject<'a, T>
    where
        P: LuaPush,
    {
        // let x = Box::new(param) ;
        let typeid = format!("{:?}_new", TypeId::of::<T>());
        // let name = CString::new(typeid.to_string()).unwrap();
        // unsafe {
        //     sys::lua_getglobal(self.lua, name.as_ptr());
        //     lua_getmetatable(self.lua, -1);
        // }
        let mut lua = Lua::from_existing_state(self.lua, false);
        match lua.query::<LuaTable, _>(typeid) {
            Some(mut table) => {
                table.set(name, param);
            }
            None => (),
        };
        self
    }

    pub fn add_method_set<P>(&mut self, name: &str, param: P) -> &mut LuaObject<'a, T>
    where
        P: LuaPush,
    {
        let typeid = format!("{:?}", TypeId::of::<T>());
        let mut lua = Lua::from_existing_state(self.lua, false);
        match lua.query::<LuaTable, _>(typeid) {
            Some(mut table) => {
                match table.query::<LuaTable, _>("__newindex") {
                    Some(mut index) => {
                        index.set(name, param);
                    }
                    None => {
                        let mut index = table.empty_table("__newindex");
                        index.set(name, param);
                    }
                };
            }
            None => (),
        };
        self
    }

    pub fn def<P>(&mut self, name: &str, param: P) -> &mut LuaObject<'a, T>
    where
        P: LuaPush,
    {
        let typeid = format!("{:?}", TypeId::of::<T>());
        let mut lua = Lua::from_existing_state(self.lua, false);
        match lua.query::<LuaTable, _>(typeid) {
            Some(mut table) => {
                match table.query::<LuaTable, _>("__index") {
                    Some(mut index) => {
                        index.set(name, param);
                    }
                    None => {
                        // let mut index = table.empty_table("__index");
                        // index.set(name, param);
                    }
                };
            }
            None => (),
        };
        self
    }

    pub fn register(
        &mut self,
        name: &str,
        func: extern "C" fn(*mut sys::lua_State) -> libc::c_int,
    ) -> &mut LuaObject<'a, T> {
        let typeid = format!("{:?}", TypeId::of::<T>());
        let mut lua = Lua::from_existing_state(self.lua, false);
        match lua.query::<LuaTable, _>(typeid) {
            Some(mut table) => {
                match table.query::<LuaTable, _>("__index") {
                    Some(mut index) => {
                        index.register(name, func);
                    }
                    None => {
                        // let mut index = table.empty_table("__index");
                        // index.register(name, func);
                    }
                };
            }
            None => (),
        };
        self
    }
}

#[macro_export]
macro_rules! add_object_field {
    ($userdata: expr, $name: ident, $t: ty, $field_type: ty) => {
        $userdata.add_method_get(&format!("{}", stringify!($name)), wmlua::function1(|obj: &mut $t| -> &$field_type { println!("aaaa"); &obj.$name }) );
        // $userdata.add_method_set(stringify!($name), wmlua::function2(|obj: &mut $t, val: $field_type| { println!("bbbb {}", val); obj.$name = val; }));
    };
}
