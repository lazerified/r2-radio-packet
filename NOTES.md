# NOTES: r2-radio-packet

## Zmiany chronologicznie
## 1. `initialize r2-radio-packet library`
- **Podział modułów** godnie ze specyfikacją moduły `crc` (obliczenia sumy kontrolnej) oraz `packet` (format, serializacja i parsowanie ramki).
- Włączono regułę `#![forbid(unsafe_code)]`. Wyklucza jakiekolwiek bloki `unsafe` w projekcie.
