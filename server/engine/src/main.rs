
// TODO
fn main() {
    // Preps python interpreter (only needs to run once, though repeat calls are negligible)
    pyo3::prepare_freethreaded_python();

    // Read env config
    // Start threadpool based on env config
    // Read state of database -- pre-load 

    // Start TCP/IP Listener for db changes (polling)
    // ...

    // Start TCP/IP Listener for Signals
    // When a new message from `bridge` is received:
    //  - Take starting data
    //  - Find corresponding Agent
    //  - Run thread with Agent Steps
    //  - 
}
