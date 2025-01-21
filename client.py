import socket
from enum import Enum
import msgpack
import json
from datetime import datetime
from typing import Any, Dict, List, Union
from uuid import UUID
import ipaddress


class ClientInputType(str, Enum):
    JoinGame = "JoinGame"
    LeaveGame = "LeaveGame"
    PauseGame = "PauseGame"
    ResumeGame = "ResumeGame"


class ClientInput:
    def __init__(self, game_id: str, player_id: str, action: ClientInputType):
        self.game_id = game_id
        self.player_id = player_id
        self.action = action

    def to_dict(self):
        return {
            "game_id": self.game_id,
            "player_id": self.player_id,
            "action": self.action.name,
        }


def format_uuid(uuid_bytes: bytes) -> str:
    """Convert UUID bytes to string format."""
    try:
        return str(UUID(bytes=uuid_bytes))
    except:
        return f"Invalid UUID: {uuid_bytes.hex()}"


def format_ip_address(ip_bytes: List[int]) -> str:
    """Format IP address from bytes."""
    try:
        return str(ipaddress.IPv4Address(bytes(ip_bytes)))
    except:
        return f"Invalid IP: {ip_bytes}"


def format_game_state(data: List) -> str:
    """Format a Game struct from list format into a readable string."""
    try:
        # Unpack the list format:
        # [game_id, players_dict, state, max_players, created_at, extra_data]
        game_id, players_dict, state, max_players, created_at, _ = data

        # Format the game state
        game = {
            "game_id": format_uuid(game_id),
            "state": state,
            "max_players": max_players,
            "created_at": created_at,
            "players": {},
        }

        # Format players
        for player_id, player_data in players_dict.items():
            # player_data format: [uuid, name, player_type, address, position]
            (
                uuid,
                name,
                player_type,
                address,
                position,
                paddle_position,
                paddle_delta,
            ) = player_data

            # Format address
            if isinstance(address, dict) and "V4" in address:
                ip, port = address["V4"]
                formatted_address = f"{format_ip_address(ip)}:{port}"
            else:
                formatted_address = str(address)

            game["players"][format_uuid(uuid)] = {
                "name": name,
                "type": player_type,
                "address": formatted_address,
                "position": position,
                "paddle_position": paddle_position,
                "paddle_delta": paddle_delta,
            }

        return json.dumps(game, indent=2)
    except Exception as e:
        return f"Error formatting game state: {e}\nRaw data: {data}"


def format_message(data: Any, prefix: str = "") -> str:
    """Format a message with timestamp and structure."""
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S.%f")[:-3]

    if isinstance(data, bytes):
        try:
            unpacked = msgpack.unpackb(data, raw=False)
            # Check if this looks like our game state list format
            if isinstance(unpacked, list) and len(unpacked) == 6:
                structure = format_game_state(unpacked)
            else:
                structure = json.dumps(unpacked, indent=2)
        except Exception as e:
            structure = f"Failed to format message: {e}\nRaw unpacked data: {data}"
    else:
        structure = json.dumps(data, indent=2)

    formatted = f"""
{prefix} @ {timestamp}
─────────────────────────────────────────────────────
{structure}
─────────────────────────────────────────────────────"""

    return formatted


def serialize_client_input(client_input: ClientInput) -> bytes:
    return msgpack.packb(client_input.to_dict(), use_bin_type=True)


def create_udp_socket() -> socket.socket:
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.settimeout(5)
    return sock


def send_message(
    sock: socket.socket, server_address: str, server_port: int, message: bytes
) -> None:
    sock.sendto(message, (server_address, server_port))
    print(format_message(message, "SENT MESSAGE"))


def listen_for_messages(sock: socket.socket) -> None:
    while True:
        try:
            data, addr = sock.recvfrom(1024)
            print(format_message(data, f"RECEIVED MESSAGE from {addr[0]}:{addr[1]}"))
        except socket.timeout:
            print("\nNo messages received. Listening again...")
            continue
        except Exception as e:
            print(f"\nError receiving message: {e}")
            continue


def main():
    server_address = "127.0.0.1"
    server_port = 34254

    sock = create_udp_socket()

    try:
        game_id = "6f59bf95-efb9-4216-9dd0-4fe0c0149513"
        player_id = "501b728f-ecc1-4193-a19e-ecbcaeff540e"
        client_input = ClientInput(game_id, player_id, ClientInputType.JoinGame)

        message = serialize_client_input(client_input)
        send_message(sock, server_address, server_port, message)

        print("\nListening for messages from the server...")
        listen_for_messages(sock)

    except KeyboardInterrupt:
        print("\nClosing connection...")
    finally:
        sock.close()


if __name__ == "__main__":
    main()
