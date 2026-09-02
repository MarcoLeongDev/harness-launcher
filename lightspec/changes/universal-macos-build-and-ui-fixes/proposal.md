# Change: Universal macOS build + engine token links + log fixes + version/check updates

## Why
The build pipeline must always produce a universal macOS binary; the engine
card/table links must include the dynamic auth token so browser opens stay
authorised; logs are concatenated into one line instead of separate lines; log
refresh/follow buttons waste vertical space; the version title is hardcoded;
the check-update message is hardcoded.

## Changes
- Universal build pipeline (Cargo.toml / build pipeline)
- Engine tab links include token dynamically
- Log tab: censor tokens, fix line breaks
- Log buttons: overlap, translucent default, opaque on hover
- Version page title: meaningful instead of hardcoded npm command
- Check update: meaningful message instead of hardcoded version
