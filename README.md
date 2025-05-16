# Decentralized Storage

This project is a decentralized storage system implemented in Rust, aiming to provide secure, distributed file storage with encryption, peer-to-peer networking, and key management.

## Project Structure

- **keys/**: Handles cryptographic key generation, storage, and management.
- **src/**: Main source code folder containing modules:
  - **encryption/**: Implements encryption and decryption functionality for secure data storage.
  - **file_system/**: Contains file operations and file system management code.
  - **key_management/**: Manages storage node keys and related access control.
  - **node/**: Implements node behavior in the decentralized network, including uploading and downloading files.
  - **p2p/**: Peer-to-peer communication protocols and networking utilities.
  - **proof_of_spacetime/**: Implements proof of spacetime protocol for verifying file storage over time.
  - **storage/**: Core logic for storage node management and file handling.
  - **storage_api_p2p/**: API endpoints for interacting with storage nodes over the P2P network.
  - Additional Rust source files under **src/** such as `api.rs`, `auth.rs`, `bsc_integration.rs`, `main.rs`, `network.rs`, `pbe_.rs`, and `storage_rs` contain supporting code for network communication, authentication, Binance Smart Chain integration, and the main program logic.

- **truffle_project/**: Smart contract development folder, indicating integration with blockchain technology (likely Ethereum).

- **.env**: Environment variables configuration for sensitive data and runtime settings.

- **Cargo.toml & Cargo.lock**: Rust package manager files managing dependencies and build information.

- **README.md**: This documentation file.

- **encrypted_key_file**: Stores encrypted keys used in the system.

## Features

- **Logging support**: Comprehensive logging integrated for debugging and monitoring.
- **File upload handling**: Enhanced mechanisms to securely upload and store files in a distributed manner.
- **User authentication**: Network communication secured with robust user authentication.
- **Key management**: Efficient and secure management of cryptographic keys.
- **Storage node management**: Nodes manage stored files, verify storage with proof of spacetime, and participate in P2P networking.
- **P2P network**: Decentralized peer-to-peer communication for file sharing and node coordination.
- **Blockchain integration**: Partially integrated with Binance Smart Chain (BSC) and Ethereum smart contracts for decentralized authentication and payments.

## Usage

1. **Setup Environment**
   - Configure `.env` file with necessary environment variables.
   - Install Rust and required dependencies via Cargo.

2. **Run Node**
   - Build the project: `cargo build --release`
   - Start the node service to join the P2P network and handle file storage.

3. **File Operations**
   - Use the API provided in `storage_api_p2p` for uploading, downloading, and managing files across nodes.
   - Encryption and decryption handled transparently via the encryption module.

4. **Authentication**
   - User authentication is managed through network communication protocols in `auth.rs`.
   - Integration with blockchain-based authentication for enhanced security.

## Dependencies

- Rust 1.79.0 or newer.
- P2P networking libraries.
- Cryptographic crates for encryption and key management.
- Blockchain SDKs and integration tools for BSC and Ethereum.

## Contributing

Contributions are welcome! Please submit pull requests or issues on the GitHub repository.

## License

This project is open source under the MIT License.

---

If you want me to add or customize anything else, just let me know!
