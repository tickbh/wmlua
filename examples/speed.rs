use wmlua::{LuaRead, LuaPush};

fn main() {
    let mut lua = wmlua::Lua::new();
    lua.openlibs();
    let val = r"
        local start = os.time();
        local sum = 0;
        for i = 0, 1000000000 do
            sum = sum + i;
        end
        print(os.time() - start);
    ";
    let _: Option<()> = lua.exec_string(val);
    println!("hello");
}