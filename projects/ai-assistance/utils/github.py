from pathlib import Path
from dotenv import load_dotenv
from os import environ

DOTENV_PATH: Path = Path(__file__).parent.parent / '.env'
load_dotenv(DOTENV_PATH, verbose=True)
GITHUB_TOKEN_NAME: str = 'GITHUB'

def get_github_token() -> str | None:
    match environ.get(GITHUB_TOKEN_NAME):
        case None:
            print(f'❌ {GITHUB_TOKEN_NAME} not present, create a .env file at the root of the repo')
            return None
        case token:
            return token
