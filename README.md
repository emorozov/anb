# Annotate branches

Simple utility that displays list of git branches along with related task title
from JIRA.

Uses configuration file `~/.config/anb.toml`:

```toml
username = "user"            # JIRA username
password = "pass"            # JIRA password
server   = "jira.domain.org" # JIRA hostname
prefix   = "TSK"             # prefix to extract task name from branch name
```