import os
import sys
import shutil
import platform
import subprocess
import urllib.request
import tarfile
import tempfile
from pathlib import Path

GITHUB_REPO = "rishiiicreates/context-grill"
VERSION = "0.1.0"
RAW_SKILL_URL = f"https://raw.githubusercontent.com/{GITHUB_REPO}/main/SKILL.md"

def is_native_binary(path_str: str) -> bool:
    try:
        p = Path(path_str).resolve()
        if p.name.endswith(".py"):
            return False
        with open(p, "rb") as f:
            magic = f.read(4)
            # Check Mach-O, ELF, or Windows PE executable signatures
            is_macho = magic in [b"\xfe\xed\xfa\xce", b"\xfe\xed\xfa\xcf", b"\xce\xfa\xed\xfe", b"\xcf\xfa\xed\xfe", b"\xca\xfe\xba\xbe"]
            is_elf = magic == b"\x7fELF"
            is_pe = magic[:2] == b"MZ"
            return is_macho or is_elf or is_pe
    except Exception:
        return False

def find_cgrill_binary() -> str | None:
    # 1. Check known binary locations first
    home = Path.home()
    candidates = [
        home / ".cargo" / "bin" / ("cgrill.exe" if sys.platform == "win32" else "cgrill"),
        home / ".cgrill" / "bin" / ("cgrill.exe" if sys.platform == "win32" else "cgrill"),
        Path("/usr/local/bin/cgrill"),
        Path("/opt/homebrew/bin/cgrill"),
        Path(__file__).resolve().parent.parent / "target" / "release" / ("cgrill.exe" if sys.platform == "win32" else "cgrill"),
    ]

    for candidate in candidates:
        if candidate.is_file() and os.access(candidate, os.X_OK) and is_native_binary(str(candidate)):
            return str(candidate)

    # 2. Check system PATH
    bin_in_path = shutil.which("cgrill")
    if bin_in_path and is_native_binary(bin_in_path):
        return bin_in_path

    return None

def download_and_install_binary() -> str | None:
    system = platform.system().lower()
    machine = platform.machine().lower()

    if system == "darwin":
        target = "aarch64-apple-darwin" if machine in ["arm64", "aarch64"] else "x86_64-apple-darwin"
    elif system == "linux":
        target = "aarch64-unknown-linux-gnu" if machine in ["arm64", "aarch64"] else "x86_64-unknown-linux-gnu"
    elif system == "windows":
        target = "x86_64-pc-windows-msvc"
    else:
        return None

    archive_name = f"cgrill-v{VERSION}-{target}.tar.gz"
    url = f"https://github.com/{GITHUB_REPO}/releases/download/v{VERSION}/{archive_name}"

    dest_dir = Path.home() / ".cgrill" / "bin"
    dest_dir.mkdir(parents=True, exist_ok=True)
    binary_dest = dest_dir / ("cgrill.exe" if system == "windows" else "cgrill")

    try:
        sys.stderr.write(f"Downloading cgrill native binary ({target})...\n")
        with urllib.request.urlopen(url) as resp:
            with tempfile.NamedTemporaryFile(delete=False, suffix=".tar.gz") as tmp:
                shutil.copyfileobj(resp, tmp)
                tmp_path = Path(tmp.name)

        with tarfile.open(tmp_path, "r:gz") as tar:
            for member in tar.getmembers():
                if member.name.endswith("cgrill") or member.name.endswith("cgrill.exe"):
                    member.name = binary_dest.name
                    tar.extract(member, path=dest_dir)
                    break
        tmp_path.unlink(missing_ok=True)
        binary_dest.chmod(0o755)
        return str(binary_dest)
    except Exception as e:
        sys.stderr.write(f"Notice: Prebuilt binary download skipped ({e}).\n")
        return None

def get_skill_content() -> str:
    local_skill = Path(__file__).resolve().parent.parent / "SKILL.md"
    if local_skill.is_file():
        return local_skill.read_text(encoding="utf-8")
    try:
        with urllib.request.urlopen(RAW_SKILL_URL, timeout=5) as resp:
            return resp.read().decode("utf-8")
    except Exception:
        return ""

def handle_skill_command(args: list[str]) -> int:
    sub = args[0] if args else "status"

    if sub == "status":
        home = Path.home()
        targets = [
            ("Antigravity (AGY)", home / ".agents" / "skills" / "context-grill" / "SKILL.md"),
            ("Claude Code", home / ".claude" / "skills" / "context-grill" / "SKILL.md"),
            ("Cursor", home / ".cursor" / "skills" / "context-grill" / "SKILL.md"),
            ("Windsurf", home / ".codeium" / "windsurf" / "skills" / "context-grill" / "SKILL.md"),
            ("OpenCode", home / ".opencode" / "skills" / "context-grill" / "SKILL.md"),
        ]
        print("\n=== CONTEXT-GRILL SKILL INSTALLATION STATUS ===")
        for name, path in targets:
            status = "Installed" if path.is_file() else "Not Installed"
            icon = "✓" if path.is_file() else "○"
            print(f"  {icon} {name}: {status}")
            if path.is_file():
                print(f"     Path: {path}")
        print()
        return 0

    elif sub == "install":
        content = get_skill_content()
        if not content:
            sys.stderr.write("Error: Could not retrieve SKILL.md\n")
            return 1

        home = Path.home()
        targets = [
            ("Antigravity (AGY)", home / ".agents" / "skills" / "context-grill" / "SKILL.md"),
            ("Claude Code", home / ".claude" / "skills" / "context-grill" / "SKILL.md"),
            ("Cursor", home / ".cursor" / "skills" / "context-grill" / "SKILL.md"),
            ("Windsurf", home / ".codeium" / "windsurf" / "skills" / "context-grill" / "SKILL.md"),
            ("OpenCode", home / ".opencode" / "skills" / "context-grill" / "SKILL.md"),
        ]
        if (home / ".gemini" / "config").is_dir():
            targets.append(("Antigravity Global Config", home / ".gemini" / "config" / "skills" / "context-grill" / "SKILL.md"))

        print("\n=== CONTEXT-GRILL SKILL INSTALLER (via uv/python) ===")
        count = 0
        for name, path in targets:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
            print(f"  ✓ {name}: Installed to {path}")
            count += 1

        print(f"\nSuccess! context-grill skill is now active across {count} platform(s).")
        print("Your agents will automatically interrogate vague prompts, inspect environment state, and guard file edits.\n")
        return 0

    return 0

def main():
    args = sys.argv[1:]

    # First check if binary exists
    binary = find_cgrill_binary()

    # If binary exists, forward everything directly
    if binary:
        try:
            res = subprocess.run([binary] + args)
            sys.exit(res.returncode)
        except KeyboardInterrupt:
            sys.exit(130)

    # If no binary, handle skill commands in pure python
    if args and args[0] == "skill":
        sys.exit(handle_skill_command(args[1:]))

    # Try downloading binary
    binary = download_and_install_binary()
    if binary:
        res = subprocess.run([binary] + args)
        sys.exit(res.returncode)

    # Fallback message
    print("context-grill: native binary not found and prebuilt release download was not available.")
    print("Run `cgrill skill install` to install the agent skill, or install Rust binary via `cargo install --git https://github.com/rishiiicreates/context-grill.git cgrill-cli`.")
    sys.exit(1)

if __name__ == "__main__":
    main()
