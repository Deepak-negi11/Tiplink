The reason your frontend is compiling so slowly is because of a cross-filesystem performance bottleneck between Windows and WSL.

I saw this in your terminal logs: `/mnt/c/Users/dayan/pnpm-lock.yaml`

This means you are running your terminal inside **WSL (Windows Subsystem for Linux)**, but your project files are physically located on your Windows `C:\` drive. 

When Next.js tries to compile, it has to read tens of thousands of files in `node_modules`. Reading Windows files (`/mnt/c/...`) from inside WSL requires passing every single read operation over a network bridge, which makes it up to **100x slower** than normal and causes it to seemingly hang indefinitely.

### How to Fix It Immediately:
You have two options:

**Option 1: Run it in a normal Windows terminal (Fastest fix)**
Close your WSL/Ubuntu terminal, open a standard **PowerShell** or **Command Prompt** (cmd) terminal, navigate to the project, and run it natively in Windows:
```cmd
cd C:\Users\dayan\Projects\TipLink\frontend
bun run dev
```

**Option 2: Move the project to Linux filesystem (Best for WSL)**
If you prefer developing in WSL, you must move the project into the Linux filesystem (e.g., your WSL `~/home` directory) instead of keeping it in `/mnt/c/...`. Compilation will be instantly fast inside WSL if the files are stored inside WSL natively.

Try Option 1 real quick — open a standard Windows PowerShell/Cmd, run `bun run dev`, and watch it compile in 2 seconds!
