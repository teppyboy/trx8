# Action 

## Available actions

### Console 

| Name   | Parameters  | Explanation                    |
| ------ | ----------- | ------------------------------ |
| `echo` | `<message>` | Echoes a string to the console |

### Subprocess 

These actions invoke subprocess to execute external commands.

Notes:
- The current working directory is the current working directory of Trx8, but can be changed by appending `--trx8-subprocess-cwd=<...>` to the parameters everywhere except the first parameter.
- Available environment variables are listed below, in addition to default environment variables and your own by appending `--trx8-subprocess-env=<env=value>`
- The command have the same privilege as Trx8, so you don't have to use `sudo` to execute them.
  

| Name          | Parameters                            | Explanation                                                    |
| ------------- | ------------------------------------- | -------------------------------------------------------------- |
| `run`         | `[<executable>, <arg1>, <arg2>, ...]` | Run an executable with arguments                               |
| `cmd`         | `<command>`                           | Run a command in `cmd.exe`                                     |
| `pwsh`        | `<command>`                           | Run a command in `powershell.exe` (not the open source `pwsh`) |
| `ti-run` (NT) | `[<executable>, <arg1>, <arg2>, ...]` | Run an executable with arguments using TrustedInstaller        |

#### Environment variables

These enviroment variables, additionally to default/passed envs to Trx8 are available in the subprocess.

| Name                   | Explanation                       |
| ---------------------- | --------------------------------- |
| `TRX8_VERSION`         | Trx8 version                      |
| `TRX8_WORKING_DIR`     | Current working directory of Trx8 |
| `TRX8_USER_CACHE_DIR`  | User cache location of Trx8       |
| `TRX8_USER_CONFIG_DIR` | User config location of Trx8      |
| `TRX8_USER_DATA_DIR`   | User data location of Trx8        |
