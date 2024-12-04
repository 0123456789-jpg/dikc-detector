# dikc-detector

Call `dikc_detector::check()` to perform checks.

## Example

```rust
use dikc_detector::check;

assert!(check().is_ok());
```

## Errors

- Errors if macOS version is equal to or newer than __`14.4`__, which is not POSIX-compliant.
- Errors if the Mac model is `MacBookPro16,1`.
