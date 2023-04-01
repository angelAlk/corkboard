use std::str::from_utf8;

mod utils;
use utils::*;

#[test]
fn double_mark() {
    ensure_new_database();

    let _feed = Miniserve::launch("./assets/sample3.rss", None);
    assert!(run_cork(&["add", "http://localhost:8080"]).status.success());

    let mark_result = run_cork(&["mark", &hash_string("azz"), &hash_string("bzz")]);
    assert!(mark_result.status.success());

    //roundabout way of saying that the output should not show the previously marked items
    let new_result = run_cork(&["new"]);
    assert!(new_result.status.success());
    let new_output = from_utf8(&new_result.stdout).expect("Could not read program output");
    assert!(!new_output.contains("azz"));
    assert!(!new_output.contains("bzz"));
    assert!(!new_output.contains(&hash_string("azz")));
    assert!(!new_output.contains(&hash_string("bzz")));
}