import socket
import threading
import select

proxy_ip = '127.0.0.1'
proxy_port = 9192


def handle_client(client_socket):
    handshake = client_socket.recv(4096)

    server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

    if handshake[0] != 0x05:
        client_socket.close()
        server_socket.close()
        print('[ERROR] Not a SOCKS5 request')
        return

    client_socket.send(b'\x05\x00')

    # Get connection request
    request = client_socket.recv(4096)
    if not request or len(request) < 7:
        client_socket.close()
        server_socket.close()
        print('[ERROR] Invalid request length')
        return

    try:
        if request[3] == 0x03:
            domain_length = request[4]
            domain = request[5:5+domain_length].decode()
            addr = socket.gethostbyname(domain)
            port = int.from_bytes(request[5+domain_length:7+domain_length], 'big')
        elif request[3] == 0x01:  # IPv4
            addr = socket.inet_ntoa(request[4:8])
            port = int.from_bytes(request[8:10], 'big')
        else:
            print("[ERROR] Unsupported address type")
            client_socket.send(b'\x05\x08\x00\x01\x00\x00\x00\x00\x00\x00')
            return
    except Exception as e:
        print(f"[ERROR] Failed to parse address: {e}")
        client_socket.send(b'\x05\x04\x00\x01\x00\x00\x00\x00\x00\x00')
        return

    try:
        server_socket.settimeout(10)
        server_socket.connect((addr, port))
        server_socket.settimeout(None)
        print(f"[INFO] Connected to {addr}:{port}")
    except Exception as e:
        print(f"[ERROR] Connection failed: {e}")
        client_socket.send(b'\x05\x04\x00\x01\x00\x00\x00\x00\x00\x00')
        return

    bind_addr = server_socket.getsockname()
    response = b'\x05\x00\x00\x01' + socket.inet_aton(bind_addr[0]) + bind_addr[1].to_bytes(2, 'big')
    client_socket.send(response)

    try:
        while True:
            r, _, _ = select.select([client_socket, server_socket], [], [], 1)
            if client_socket in r:
                data = client_socket.recv(4096)
                if not data:
                    break
                server_socket.send(data)
            if server_socket in r:
                data = server_socket.recv(4096)
                if not data:
                    break
                client_socket.send(data)
    except Exception as e:
        print(f"[ERROR] Data forwarding failed: {e}")
    finally:
        client_socket.close()
        server_socket.close()


def proxy_server():
    proxy = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    proxy.bind((proxy_ip, proxy_port))
    proxy.listen(5)

    print(f'[INFO] Proxy server listening on {proxy_ip}:{proxy_port}')

    while True:
        client_socket, client_addr = proxy.accept()
        print(f'[INFO] Received connection from {client_addr[0]}:{client_addr[1]}')
        client_handler = threading.Thread(target=handle_client, args=(client_socket,))
        client_handler.start()


if __name__ == '__main__':
    proxy_server()
