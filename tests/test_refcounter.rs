use k9::assert_equal;
use unique_pointer::RefCounter;

#[test]
fn test_refcounter_incr_decr_read() {
    let mut counter = RefCounter::new();
    assert_equal!(counter.read(), 1);
    counter.incr();
    assert_equal!(counter.read(), 2);
    counter.incr();
    assert_equal!(counter.read(), 3);
    {
        let mut clone = counter.clone();
        clone.incr();
        assert_equal!(counter.read(), 4);
        assert_equal!(clone.read(), 4);
    }
    assert_equal!(counter.read(), 4);
    counter.decr();
    assert_equal!(counter.read(), 3);
    counter.decr();
    assert_equal!(counter.read(), 2);
    counter.decr();
    assert_equal!(counter.read(), 1);
    counter.decr();
    assert_equal!(counter.read(), 0);
    counter.decr();
    assert_equal!(counter.read(), 0);
}
#[test]
fn test_refcounter_deref() {
    let mut counter = RefCounter::new();
    assert_equal!(counter.read(), 1);
    counter.incr();
    assert_equal!(counter.read(), 2);
    counter.incr();
    assert_equal!(counter.read(), 3);
    let refs: usize = *counter;
    assert_equal!(refs, 3);
}
#[test]
fn test_refcounter_add_assign() {
    let mut counter = RefCounter::new();
    assert_equal!(counter.read(), 1);
    counter += 2;
    assert_equal!(counter.read(), 3);
    counter -= 1;
    assert_equal!(counter.read(), 2);
    counter -= 1;
    assert_equal!(counter.read(), 1);
    counter += 1;
    let refs: usize = *counter;
    assert_equal!(refs, 2);
}
