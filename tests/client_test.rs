use rsc::client::Client;

#[test]
fn test_ping() {
    let s = Client::hello();
    assert_eq!(s, "Hello")
}