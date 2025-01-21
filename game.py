import json
import pygame
import socket
import msgpack
from enum import Enum
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
import requests

# Constants for the server URLs
SERVER_URL = "http://localhost:3000"
CREATE_GAME_URL = f"{SERVER_URL}/game"
JOIN_GAME_URL = f"{SERVER_URL}/game/{{game_id}}/join"


# Function to create a new game and return the game ID
def create_game() -> str:
    response = requests.post(CREATE_GAME_URL)
    if response.status_code == 200:
        game_data = response.json()
        return game_data["id"]
    else:
        raise Exception(f"Failed to create game: {response.status_code}")


# Function to join a game and return the player ID
def join_game(game_id: str) -> str:
    headers = {
        "Content-Type": "application/json",
    }
    response = requests.post(
        JOIN_GAME_URL.format(game_id=game_id), headers=headers, json={}
    )
    if response.status_code == 200:
        player_data = response.json()
        return player_data["id"]
    else:
        raise Exception(
            f"Failed to join game: {response.status_code}, Response: {response.text}"
        )


# Automatically fetch game_id and player_id
game_id_str = create_game()
player_id = join_game(game_id_str)

print(f"Game ID: {game_id_str}")
print(f"Player ID: {player_id}")


class Direction(str, Enum):
    Positive = "Positive"
    Negative = "Negative"


class ClientInputType(str, Enum):
    JoinGame = "JoinGame"
    LeaveGame = "LeaveGame"
    PauseGame = "PauseGame"
    ResumeGame = "ResumeGame"
    StartGame = "StartGame"
    MovePaddle = "MovePaddle"


@dataclass
class Player:
    name: str
    score: int
    position: Optional[str]
    paddle_pos: float
    paddle_delta: float


@dataclass
class Ball:
    position: Tuple[float, float]  # (x, y)
    velocity: Tuple[float, float]  # (vx, vy)
    radius: float


@dataclass
class GameState:
    players: Dict[str, Player]
    ball: Optional[Ball]
    current_player_id: str

    def update(self, new_players: Dict, new_ball: Optional[Ball]):
        self.players.update(new_players)
        if new_ball:
            self.ball = new_ball


class ClientInput:
    def __init__(
        self,
        game_id: str,
        player_id: str,
        action: ClientInputType,
        direction: Optional[Direction] = None,
    ):
        self.game_id = game_id
        self.player_id = player_id
        self.action = action
        self.direction = direction

    def to_dict(self):
        action_dict = {"type": self.action.value}
        if self.action == ClientInputType.MovePaddle and self.direction:
            action_dict["data"] = self.direction.value
        return {
            "game_id": game_id_str,
            "player_id": self.player_id,
            "action": action_dict,
        }


def handle_continuous_input(
    keys,
    sock: socket.socket,
    server_address: str,
    server_port: int,
    game_id: str,
    player_id: str,
):
    """Handle continuous key press state"""
    direction = None

    if keys[pygame.K_UP] or keys[pygame.K_RIGHT]:
        direction = Direction.Positive
    elif keys[pygame.K_DOWN] or keys[pygame.K_LEFT]:
        direction = Direction.Negative

    if direction:
        client_input = ClientInput(
            game_id, player_id, ClientInputType.MovePaddle, direction
        )
        message = serialize_client_input(client_input)
        send_message(sock, server_address, server_port, message)


# Initialize Pygame
pygame.init()

# Constants
WINDOW_SIZE = 800
PADDLE_LENGTH = 100
PADDLE_WIDTH = 10
CENTER_CIRCLE_RADIUS = 50
BACKGROUND_COLOR = (0, 0, 0)  # Black
PADDLE_COLOR = (255, 255, 255)  # White
MY_PADDLE_COLOR = (0, 255, 0)  # Green for current player's paddle
CENTER_COLOR = (50, 50, 50)  # Dark gray
TEXT_COLOR = (200, 200, 200)  # Light gray
BALL_COLOR = (255, 0, 0)  # Red

# Create the window
screen = pygame.display.set_mode((WINDOW_SIZE, WINDOW_SIZE))
pygame.display.set_caption("Quadropong Visualizer")
clock = pygame.time.Clock()


def serialize_client_input(client_input: ClientInput) -> bytes:
    data = client_input.to_dict()
    return msgpack.packb(data, use_bin_type=True)


def game_to_screen_coords(x: float, y: float) -> Tuple[int, int]:
    """Convert game coordinates (0-10) to screen coordinates (0-WINDOW_SIZE)"""
    screen_x = int(x * WINDOW_SIZE / 10)
    screen_y = int(y * WINDOW_SIZE / 10)
    return screen_x, screen_y


def draw_paddle(
    position: str,
    paddle_pos: float,
    player_name: str,
    score: int,
    is_current_player: bool,
    paddle_width: float,
):
    paddle_length = abs(paddle_width) * (WINDOW_SIZE / 10)  # Scale to screen size
    PADDLE_THICKNESS = 10  # Constant thickness for all paddles
    EDGE_SPACING = 0.5 * (WINDOW_SIZE / 10)  # Space between paddle and screen edge

    # Calculate the paddle's screen position based on 0-10 range
    if position == "Top":
        x = paddle_pos * WINDOW_SIZE / 10
        y = EDGE_SPACING
        rect = pygame.Rect(
            int(x - paddle_length / 2),
            int(y - PADDLE_THICKNESS / 2),
            int(paddle_length),
            PADDLE_THICKNESS,
        )
    elif position == "Bottom":
        x = paddle_pos * WINDOW_SIZE / 10
        y = WINDOW_SIZE - EDGE_SPACING
        rect = pygame.Rect(
            int(x - paddle_length / 2),
            int(y - PADDLE_THICKNESS / 2),
            int(paddle_length),
            PADDLE_THICKNESS,
        )
    elif position == "Left":
        x = EDGE_SPACING
        y = paddle_pos * WINDOW_SIZE / 10
        rect = pygame.Rect(
            int(x - PADDLE_THICKNESS / 2),
            int(y - paddle_length / 2),
            PADDLE_THICKNESS,
            int(paddle_length),
        )
    elif position == "Right":
        x = WINDOW_SIZE - EDGE_SPACING
        y = paddle_pos * WINDOW_SIZE / 10
        rect = pygame.Rect(
            int(x - PADDLE_THICKNESS / 2),
            int(y - paddle_length / 2),
            PADDLE_THICKNESS,
            int(paddle_length),
        )

    color = MY_PADDLE_COLOR if is_current_player else PADDLE_COLOR
    pygame.draw.rect(screen, color, rect)

    # Draw player name and score
    font = pygame.font.Font(None, 24)
    text = font.render(f"{player_name}: {score}", True, TEXT_COLOR)

    # Adjust text positions based on paddle position
    if position == "Top":
        text_pos = (int(x - text.get_width() / 2), int(y + PADDLE_THICKNESS + 10))
    elif position == "Bottom":
        text_pos = (int(x - text.get_width() / 2), int(y - PADDLE_THICKNESS - 30))
    elif position == "Left":
        text_pos = (int(x + PADDLE_THICKNESS + 10), int(y - text.get_height() / 2))
    else:  # Right
        text_pos = (
            int(x - text.get_width() - PADDLE_THICKNESS - 10),
            int(y - text.get_height() / 2),
        )

    screen.blit(text, text_pos)


def draw_ball(ball: Ball):
    """Draw the ball on the screen"""
    # Convert ball position from game coordinates (0-10) to screen coordinates
    screen_x, screen_y = game_to_screen_coords(ball.position[0], ball.position[1])

    # Convert radius from game units to screen pixels
    screen_radius = int(ball.radius * WINDOW_SIZE / 10)

    pygame.draw.circle(screen, BALL_COLOR, (screen_x, screen_y), screen_radius)


def create_udp_socket() -> socket.socket:
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.settimeout(0.1)
    return sock


def send_message(
    sock: socket.socket, server_address: str, server_port: int, message: bytes
) -> None:
    sock.sendto(message, (server_address, server_port))


def main():
    global game_id_str, player_id
    server_address = "127.0.0.1"
    server_port = 34254

    # Initialize game state
    game_state = GameState({}, None, player_id)

    sock = create_udp_socket()
    client_input = ClientInput(game_id_str, player_id, ClientInputType.JoinGame)
    message = serialize_client_input(client_input)
    send_message(sock, server_address, server_port, message)
    print(f"Joining game {game_id_str} as player {player_id}")

    running = True
    while running:
        # Handle pygame events
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False
            elif event.type == pygame.KEYDOWN:
                if event.key == pygame.K_ESCAPE:
                    running = False

        keys = pygame.key.get_pressed()
        handle_continuous_input(
            keys, sock, server_address, server_port, game_id_str, player_id
        )

        # Clear screen
        screen.fill(BACKGROUND_COLOR)

        try:
            # Receive game state
            data, addr = sock.recvfrom(1024)
            unpacked = msgpack.unpackb(data, raw=False)

            if isinstance(unpacked, list) and len(unpacked) == 6:
                game_id, state, created_at, started_at, ball_data, players_dict = (
                    unpacked
                )
                # Create Ball object directly from the received coordinates (already in 0-10 range)
                if isinstance(ball_data, list) and len(ball_data) == 3:
                    ball_position, ball_velocity, ball_radius = ball_data
                    ball = Ball(
                        position=(ball_position[0], ball_position[1]),
                        velocity=(ball_velocity[0], ball_velocity[1]),
                        radius=ball_radius,
                    )
                    game_state.update(players_dict, ball)

        except socket.timeout:
            pass  # No data received
        except Exception as e:
            print(f"Error: {e}")

        # Draw paddles using current game state
        for player_id_key, player_data in game_state.players.items():
            (
                name,
                score,
                position,
                paddle_pos,
                paddle_delta,
                paddle_width,
            ) = player_data
            if position:
                is_current_player = player_id_key == id
                draw_paddle(
                    position,
                    paddle_pos,  # Use paddle_pos directly since it's already in 0-10 range
                    name,
                    score,
                    is_current_player,
                    paddle_width,
                )

        # Draw ball
        if game_state.ball:
            draw_ball(game_state.ball)

        # Update display
        pygame.display.flip()
        clock.tick(120)  # Cap frame rate at 120 FPS

    # Clean up
    print("\nClosing connection...")
    client_input = ClientInput(game_id_str, player_id, ClientInputType.LeaveGame)
    message = serialize_client_input(client_input)
    send_message(sock, server_address, server_port, message)

    pygame.quit()
    sock.close()


if __name__ == "__main__":
    main()
