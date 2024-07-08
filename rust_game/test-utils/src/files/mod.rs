use std::{
    env,
    fs,
    panic::{
        catch_unwind,
        UnwindSafe,
    },
    path::{
        Path,
        PathBuf,
    },
};

use rand::Rng;

fn create_test_tmp_dir_path() -> PathBuf {
    let mut rng = rand::thread_rng();
    let rand = rng.gen_range(0..u32::MAX);

    let pwd = env::var("PWD").unwrap();
    let mut path_buf = PathBuf::from(pwd);
    path_buf.push("target");
    path_buf.push("test-tmp");
    path_buf.push(format!("id-{rand}"));

    return path_buf;
}

pub fn with_test_temp_dir(test: impl FnOnce(&Path) + UnwindSafe) {
    let path = create_test_tmp_dir_path();

    fs::create_dir_all(&path).unwrap();

    let test_result = catch_unwind(|| {
        test(path.as_path());
    });

    fs::remove_dir_all(&path).unwrap();

    test_result.unwrap();
}
