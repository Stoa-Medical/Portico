This is the desktop app.

It has the option to 1) connect to a server to manage, or 2) run indepdently with an embedded db

Proposed stack:
- Tauri
    - Vanilla JS frontend (alternatively try Slint. Just want something simple to understand)
    - Rust backend
        - Python interpreter with PyO3

Requirements:
1. Write data mappings in Python
2. Save data mappings in Python
3. Set listeners to incoming ports
4. Reroute data to outgoing ports
5. Manages public/private keypair for encrypting data
6. Manages signing key for signing data
