test_func(0) -> i32 {
    i32 i;
    i = 10;
    return i;
}

wasm:bytecode:call:after /
    target_fn_type == "import" &&
    target_imp_module == "ic0" &&
    target_imp_name == "add"
/ {
    i32 t;
    t = test_func(t);
}