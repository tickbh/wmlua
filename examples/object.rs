use std::{any::TypeId, ffi::CString};

use wmlua::{add_object_field, lua_State, object_impl, LuaObject, LuaPush, LuaRead};

struct Xx {
    kk: String,
    nn: String,
}

impl  Default for Xx {
    fn default() -> Self {
        Self { kk: "Default::default()".to_string(), nn: Default::default() }
    }
}

object_impl!(Xx);

fn main() {
    // let mut xx = Xx { kk: String::new(), nn: String::new() };
    // test!(xx, kk);
    // println!("kkk = {:?} nn = {:?}", xx.kk, xx.nn);
    let mut lua = wmlua::Lua::new();
    let mut object = LuaObject::<Xx>::new(lua.state(), "CCCC");
    object.create();
    add_object_field!(object, kk, Xx, String);
    // object.def("xxx", wmlua::function1(|obj: &mut Xx| "sss".to_string()));

    object.add_method_get("xxx", wmlua::function1(|obj: &mut Xx| "sss is xxx".to_string()));
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
        local v = CCCC();
        print(\"vvvvv\", v:xxx())
        print(\"kkkk\", v.kk)
        v.kk = \"aa\";
        print(\"ccccc\", v.kk)
        print(\"vvvvv\", v:xxx())
        --print(v.kk(v))
    ";

    // #v = CCCC();
    // #print(v);
    // #v.kk = \"aaaaa\";
    // #print(v.kk);
    let _: Option<()> = lua.exec_string(val);
    println!("hello");
}