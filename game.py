import pygame
import socket
import msgpack
from enum import Enum
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
import time

game_id_str = "f77231d4-9393-441f-8084-1c71951c355d"
player_id = "d0694a77-80ce-4cbc-b351-8fc3d4bf9e34"


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
    id: str
    name: str
    score: int
    address: Optional[str]
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

    if keys[pygame.K_UP] or keys[pygame.K_LEFT]:
        direction = Direction.Positive
        print("Positive direction (Up/Left)")
    elif keys[pygame.K_DOWN] or keys[pygame.K_RIGHT]:
        direction = Direction.Negative
        print("Negative direction (Down/Right)")

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
    print("Serialized data:", data)  # Debugging
    return msgpack.packb(data, use_bin_type=True)


def draw_paddle(
    position: str,
    paddle_pos: float,
    player_name: str,
    score: int,
    is_current_player: bool,
):
    """Draw a paddle and player info"""
    x, y = 0, 0

    # Clamp paddle position to valid range
    paddle_pos = max(-10, min(10, paddle_pos))

    # Use server-provided paddle position directly
    if position == "Top":
        paddle_screen_pos = (paddle_pos + 10) * (WINDOW_SIZE - PADDLE_LENGTH) / 20
        x = int(paddle_screen_pos + PADDLE_LENGTH / 2)
        y = PADDLE_WIDTH / 2
        rect = pygame.Rect(
            x - PADDLE_LENGTH / 2, y - PADDLE_WIDTH / 2, PADDLE_LENGTH, PADDLE_WIDTH
        )
    elif position == "Bottom":
        paddle_screen_pos = (paddle_pos + 10) * (WINDOW_SIZE - PADDLE_LENGTH) / 20
        x = int(paddle_screen_pos + PADDLE_LENGTH / 2)
        y = WINDOW_SIZE - PADDLE_WIDTH / 2
        rect = pygame.Rect(
            x - PADDLE_LENGTH / 2, y - PADDLE_WIDTH / 2, PADDLE_LENGTH, PADDLE_WIDTH
        )
    elif position == "Left":
        paddle_screen_pos = (paddle_pos + 10) * (WINDOW_SIZE - PADDLE_LENGTH) / 20
        x = PADDLE_WIDTH / 2
        y = int(paddle_screen_pos + PADDLE_LENGTH / 2)
        rect = pygame.Rect(
            x - PADDLE_WIDTH / 2, y - PADDLE_LENGTH / 2, PADDLE_WIDTH, PADDLE_LENGTH
        )
    elif position == "Right":
        paddle_screen_pos = (paddle_pos + 10) * (WINDOW_SIZE - PADDLE_LENGTH) / 20
        x = WINDOW_SIZE - PADDLE_WIDTH / 2
        y = int(paddle_screen_pos + PADDLE_LENGTH / 2)
        rect = pygame.Rect(
            x - PADDLE_WIDTH / 2, y - PADDLE_LENGTH / 2, PADDLE_WIDTH, PADDLE_LENGTH
        )

    color = MY_PADDLE_COLOR if is_current_player else PADDLE_COLOR
    pygame.draw.rect(screen, color, rect)

    # Draw player name and score
    font = pygame.font.Font(None, 24)
    text = font.render(f"{player_name}: {score}", True, TEXT_COLOR)

    # Position text based on paddle location
    if position == "Top":
        text_pos = (x - text.get_width() / 2, y + 20)
    elif position == "Bottom":
        text_pos = (x - text.get_width() / 2, y - 40)
    elif position == "Left":
        text_pos = (x + 20, y - text.get_height() / 2)
    else:  # Right
        text_pos = (x - text.get_width() - 20, y - text.get_height() / 2)

    screen.blit(text, text_pos)


def draw_ball(ball: Ball):
    """Draw the ball on the screen"""
    x = int((ball.position[0] + 10) * (WINDOW_SIZE / 20))
    y = int((ball.position[1] + 10) * (WINDOW_SIZE / 20))

    pygame.draw.circle(
        screen, BALL_COLOR, (x, y), int(WINDOW_SIZE * (ball.radius / 20))
    )


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
    last_time = time.time()
    while running:
        current_time = time.time()
        delta_time = current_time - last_time
        last_time = current_time
        print(f"Frame time: {delta_time:.4f} seconds")

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
            # print("Received data:", unpacked)  # Debugging

            if isinstance(unpacked, list) and len(unpacked) == 7:
                game_id, players_dict, state, max_players, created_at, _, ball_data = (
                    unpacked
                )

                # Unpack ball data
                if isinstance(ball_data, list) and len(ball_data) == 3:
                    ball_position, ball_velocity, ball_radius = ball_data
                    ball = Ball(
                        position=(ball_position[0], ball_position[1]),  # (x, y)
                        velocity=(ball_velocity[0], ball_velocity[1]),  # (vx, vy)
                        radius=ball_radius,
                    )
                    game_state.update(players_dict, ball)
                else:
                    print("Invalid ball data format:", ball_data)

        except socket.timeout:
            pass  # No data received
        except Exception as e:
            print(f"Error: {e}")

        # Draw paddles using current game state
        for player_id_key, player_data in game_state.players.items():
            id, name, score, address, position, paddle_pos, paddle_delta = player_data
            if position:
                is_current_player = player_id_key == id
                draw_paddle(position, paddle_pos, name, score, is_current_player)

        # Draw ball
        if game_state.ball:
            draw_ball(game_state.ball)

        # Update display
        pygame.display.flip()
        clock.tick(60)  # Cap frame rate at 120 FPS

    # Clean up
    print("\nClosing connection...")
    client_input = ClientInput(game_id_str, player_id, ClientInputType.LeaveGame)
    message = serialize_client_input(client_input)
    send_message(sock, server_address, server_port, message)

    pygame.quit()
    sock.close()


if __name__ == "__main__":
    main()
