//-----------------------------------------------------------------------
//
//  mod parser  TEST
//
//-----------------------------------------------------------------------

use crate::parser::{expression::parse, Status};

#[test]
fn test_parse_expr_lit() {
    let rules = rules! {"main" => RuleInfo{expr: lit!("aaa"), descr: None} };
    let status_init = Status::init("aaaaaaaaaaaaaaaa", &rules);

    let (status, _) = parse(status_init).ok().unwrap();
    assert!(status.pos.col == 3);
    assert!(status.pos.n == 3);
    assert!(status.pos.row == 0);
}

#[test]
fn test_parse_expr_and_ok() {
    let rules = rules! {"main" => RuleInfo{expr: and![lit!("aa"), and![lit!("bb"), lit!("cc")]], descr: None} };
    let status_init = Status::init("aabbcc", &rules);

    let (status, _) = parse(status_init).ok().unwrap();
    assert_eq!(status.pos.col, 6);
    assert_eq!(status.pos.n, 6);
    assert_eq!(status.pos.row, 0);
}

#[test]
fn test_parse_expr_or_ok() {
    let rules = rules! {"main" => RuleInfo{expr: or![lit!("bb"), and![lit!("aa"), lit!("bb")]], descr: None} };
    let status_init = Status::init("aabb", &rules);

    let (status, _) = parse(status_init).ok().unwrap();
    assert_eq!(status.pos.col, 4);
    assert_eq!(status.pos.n, 4);
    assert_eq!(status.pos.row, 0);
}

#[test]
fn test_parse_expr_not_ok() {
    let rules = rules! {"main" => RuleInfo{expr: not!(lit!("bb")), descr: None} };
    let status_init = Status::init("aa", &rules);

    let (status, _) = parse(status_init).ok().unwrap();
    assert_eq!(status.pos.col, 0);
    assert_eq!(status.pos.n, 0);
    assert_eq!(status.pos.row, 0);
}

#[test]
fn test_parse_expr_repeat_ok() {
    let rules = rules! {"main" => RuleInfo{expr: rep![lit!("aa"), 3], descr: None} };
    {
        let status_init = Status::init("aaaaaa", &rules);

        let (status, _) = parse(status_init).ok().unwrap();
        assert_eq!(status.pos.col, 6);
        assert_eq!(status.pos.n, 6);
        assert_eq!(status.pos.row, 0);
    }

    // {
    //     let status_init = Status::init("aaaaaa", rep![lit!("aa"), 0, 3]);

    //     let result = parse(status_init).ok().unwrap();
    //     assert_eq!(result.status.pos.col, 6);
    //     assert_eq!(result.status.pos.n, 6);
    //     assert_eq!(result.status.pos.row, 0);
    // }
}
