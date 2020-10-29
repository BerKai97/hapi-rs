extern crate hapi_rs as he;


fn main_() -> he::Result<()> {
    let cook_options = he::CookOptions::default();
    let session = he::Session::new_in_process()?;
    let mut res = he::Initializer::new(session.clone());
    res.set_cook_options(&cook_options);
    res.set_houdini_env_files(&["/foo", "/bar"]);
    res.initialize()?;
    dbg!(session);
    Ok(())
}

fn main() {
    if let Err(e) = main_() {
        eprintln!("{}", e)
    }
}