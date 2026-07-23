use backup::commands::uninstall::{execute_uninstall_plan, perform_uninstall};
use backup::commands::update::execute_update_check;

#[test]
fn test_uninstall_plan() {
    let plan = execute_uninstall_plan();
    assert!(plan.contains("/usr/local/sbin/backup"));
    assert!(plan.contains("/etc/backup"));
}

#[test]
fn test_perform_uninstall_with_yes() {
    let res = perform_uninstall(true).unwrap();
    assert!(res.contains("Uninstalled"));
}

#[test]
fn test_perform_uninstall_non_interactive_without_yes_fails() {
    let res = perform_uninstall(false);
    assert!(res.is_err());
}

#[test]
fn test_update_check() {
    let result = execute_update_check("1.0.0").unwrap();
    assert!(result.contains("1.0.0"));
    assert!(result.contains("up to date"));
}
