use std::{any::TypeId, ffi::CString};

use wmlua::{add_object_field, lua_State, LuaObject, LuaPush, LuaRead};

struct Xx {
    kk: String,
    nn: String,
}

impl  Default for Xx {
    fn default() -> Self {
        Self { kk: "Default::default()".to_string(), nn: Default::default() }
    }
}


impl<'a> LuaRead for &'a mut Xx {
    fn lua_read_with_pop_impl(lua: *mut lua_State, index: i32, _pop: i32) -> Option<&'a mut Xx> {
        wmlua::userdata::read_userdata(lua, index)
    }
}

impl LuaPush for Xx {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        unsafe {
            let obj = Box::into_raw(Box::new(self));
            wmlua::userdata::push_lightuserdata(&mut *obj, lua, |_| {});
            let typeid = CString::new(format!("{:?}", TypeId::of::<Xx>())).unwrap();
            wmlua::lua_getglobal(lua, typeid.as_ptr());
            if wmlua::lua_istable(lua, -1) {
                wmlua::lua_setmetatable(lua, -2);
            } else {
                wmlua::lua_pop(lua, 1);
            }
            1
        }
    }
}
fn main() {
    // let mut xx = Xx { kk: String::new(), nn: String::new() };
    // test!(xx, kk);
    // println!("kkk = {:?} nn = {:?}", xx.kk, xx.nn);
    let mut lua = wmlua::Lua::new();
    let mut object = LuaObject::<Xx>::new(lua.state(), "CCCC");
    object.create();
    add_object_field!(object, kk, Xx, String);
    lua.openlibs();
    // let val = "
    //     print(\"ccc\");
    //     print(type(CCCC));
    //     let vv = CCCC();
    //     vv.kk = \"aaa\";
    //     print(vv.kk);
    // ";

    let val = "
        print(aaa);
        print(\"cccxxxxxxxxxxxxxxx\");
        print(type(CCCC));
        v = CCCC();
        print(\"kkkk\", v.kk)
        v.kk = \"aa\";
        print(\"ccccc\", v.kk)
        --print(v.kk(v))
    ";

    // #v = CCCC();
    // #print(v);
    // #v.kk = \"aaaaa\";
    // #print(v.kk);
    let _: Option<()> = lua.exec_string(val);
    println!("hello");
}