// rust-hidden-tests/hw00_add/tests/hidden_basic.rs
use hw00_add::add;
use hw00_add::add_if_nonnegative;
use hw00_add::max;

#[test]
fn basic_add_small_numbers() {
    assert_eq!(add(2, 3), 5);
    assert_eq!(add(0, 0), 0);
    assert_eq!(add(-4, 9), 5);
}

#[test]
fn basic_add_with_negatives() {
    assert_eq!(add(-10, -5), -15);
    assert_eq!(add(-1, 1), 0);
}

#[test]
fn basic_add_identity_zero() {
    for x in [-100, -1, 0, 1, 100] {
        assert_eq!(add(x, 0), x);
        assert_eq!(add(0, x), x);
    }
}

#[test]
fn basic_max_picks_larger_positive() {
    assert_eq!(max(3, 7), 7);
    assert_eq!(max(42, 1), 42);
}

#[test]
fn basic_max_with_negatives_and_zero() {
    assert_eq!(max(-5, -2), -2);
    assert_eq!(max(0, -1), 0);
    assert_eq!(max(-1, 0), 0);
}

#[test]
fn basic_max_equal_values() {
    assert_eq!(max(10, 10), 10);
    assert_eq!(max(-3, -3), -3);
}

#[test]
fn basic_impossible() {
    assert_eq!(1, 2);
}

// CORNER CASES for add_if_non_negative
#[test]
fn corner_both_nonnegative_returns_sum() {
    assert_eq!(add_if_nonnegative(0, 0), Some(0));
    assert_eq!(add_if_nonnegative(5, 7), Some(12));
    assert_eq!(add_if_nonnegative(100, 0), Some(100));
}

#[test]
fn corner_any_negative_returns_none() {
    assert_eq!(add_if_nonnegative(-1, 0), None);
    assert_eq!(add_if_nonnegative(0, -1), None);
    assert_eq!(add_if_nonnegative(-3, -4), None);
}

#[test]
fn corner_larger_values_still_ok() {
    // Not testing overflow here; just sanity checks.
    assert_eq!(add_if_nonnegative(2_000_000_000, 100), Some(2_000_000_100));
}
