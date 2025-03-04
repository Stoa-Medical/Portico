"""
A test TCP/IP server for local integration testing (i.e. independent of Rust `engine`)
"""

import os
import socket
import threading
import time

from dotenv import load_dotenv


class TcpIpServer:
    def __init__(self, host, port):
        self.host = host
        self.port = port
        self._running = False
        self._server_thread = None

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
            print(f"Server listening on {self.host}:{self.port}")
            while self._running:
                try:
                    client, addr = sock.accept()
                    print(f"Connection from {addr}")
                    data = client.recv(4096)
                    if data:
                        print(f"Received data: {data.decode()}")
                    client.close()
                except Exception as e:
                    print(f"Error: {e}")
                    break

    def stop(self):
        self._running = False
        if self._server_thread:
            self._server_thread.join(timeout=1.0)


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
