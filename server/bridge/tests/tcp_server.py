"""
A test TCP/IP server for local integration testing (i.e. independent of Rust `engine`)
"""

import os
import socket
import threading
import time
import json

from dotenv import load_dotenv


class TcpIpServer:
    def __init__(self, host, port):
        self.host = host
        self.port = port
        self._running = False
        self._server_thread = None
        self._conn_history = []

    def start(self):
        self._running = True
        self._server_thread = threading.Thread(target=self._run_server)
        self._server_thread.daemon = True
        self._server_thread.start()
        time.sleep(0.1)  # Give server time to start

    def _run_server(self):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.bind((self.host, self.port))
            sock.listen(1)
            sock.settimeout(1.0)  # Add timeout to allow checking _running flag
            print(f"Server listening on {self.host}:{self.port}")
            
            while self._running:
                try:
                    client, addr = sock.accept()
                    print(f"Connection from {addr}")
                    self._conn_history.append(addr)
                    
                    # Handle client in a separate thread to keep server responsive
                    client_thread = threading.Thread(
                        target=self._handle_client,
                        args=(client, addr)
                    )
                    client_thread.daemon = True
                    client_thread.start()
                    
                except socket.timeout:
                    # This is expected due to the timeout we set
                    continue
                except Exception as e:
                    print(f"Error accepting connection: {e}")
                    break

    def _handle_client(self, client, addr):
        """Handle communication with a single client"""
        try:
            client.settimeout(30.0)  # Set a reasonable timeout
            
            while self._running:
                try:
                    data = client.recv(4096)
                    if not data:
                        print(f"Client {addr} disconnected")
                        break
                        
                    print(f"Received data from {addr}: {data.decode()}")
                    
                    # Send a response back (optional)
                    response = {"status": "ok", "message": "received"}
                    response_data = json.dumps(response).encode("utf-8")
                    client.sendall(response_data)
                    
                except socket.timeout:
                    # Check if we should still be running
                    if not self._running:
                        break
                    continue
                    
        except Exception as e:
            print(f"Error handling client {addr}: {e}")
        finally:
            client.close()
            print(f"Closed connection with {addr}")

    def stop(self):
        self._running = False
        if self._server_thread:
            self._server_thread.join(timeout=1.0)
        print(f"Dropping connections: {self._conn_history}")


if __name__ == "__main__":
    # Read environment variables
    load_dotenv()
    HOST = os.getenv("ENGINE_HOST_NAME", "localhost")
    PORT = int(os.getenv("ENGINE_PORT", 8888))

    # Create and start the server
    server = TcpIpServer(HOST, PORT)
    server.start()

    # Keep the server running and print received data
    try:
        while True:
            time.sleep(1)
            # Server prints received data in _run_server() method
            # The data is expected to be JSON from the bridge service
    except KeyboardInterrupt:
        print("\nShutting down server...")
        server.stop()
