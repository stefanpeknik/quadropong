# Quadropong

Quadropong is a Rust-based game project. This README provides instructions on how to set up and run the project.

## Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)

## Setup

1. **Clone the repository:**

    ```sh
    git clone <repository-url>
    cd quadropong
    ```

2. **Build the project:**

    ```sh
    cargo build
    ```

3. **Run the project:**

    To run the client:

    ```sh
    cargo run --bin client
    ```

    To run the server:

    ```sh
    cargo run --bin server
    ```

### Environment variables

Optionally, it could be useful to change the API URL and/or the socket address.
That can be done by creating a file [`config.toml`](https://doc.rust-lang.org/cargo/reference/config.html)
in project's root directory and specifying those environment variables,
otherwise they default to localhost addresses:

```conf
[env]
API_URL='...'       # REST API address for client to communicate with the server
SOCKET_ADDR='...'   # UDP socket address that server listens on for client updates 
```

## Testing

To run the tests, use the following command:

```sh
cargo test
```

## Logging

### Client

The logs are saved to a file in [user's local data directory](https://docs.rs/dirs/6.0.0/dirs/fn.data_local_dir.html).

### Server

The logs are printed to a standard output and also to a file in a current directory.
