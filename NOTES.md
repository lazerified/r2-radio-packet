# NOTES: r2-radio-packet

# TODO: pytania z pliku `R2_radio_packet_crc.md`

## Zmiany chronologicznie
## 1. `initialize r2-radio-packet library`
- **Podział modułów** godnie ze specyfikacją moduły `crc` (obliczenia sumy kontrolnej) oraz `packet` (format, serializacja i parsowanie ramki).
- Włączono regułę `#![forbid(unsafe_code)]`. Wyklucza jakiekolwiek bloki `unsafe` w projekcie.

## 2. `implement crc16_ccitt_false with tests`
Algorytm `crc16_ccitt_false` oblicza resztę z dzielenia przez wielomian `0x1021` ($x^{16} + x^{12} + x^5 + 1$).
- initial value / rejestr początkowy `0xFFFF`.
- MSB-first. Każdy kolejny bajt jest przesuwany o 8 bitów w lewo (`byte << 8`) i łączony z rejestrem operacją XOR, przez to trafia na najstarszą pozycję.
- Pętla 8 bitów, tj. jeśli bit 15 rejestru wynosi 1 (`crc & 0x8000 != 0`), następuje przesunięcie w lewo o 1 bit i reszta z dzielenia z wielomianem, a w praktyce XOR bo dzialamy w ciele binarnym (`(crc << 1) ^ 0x1021`). W przeciwnym razie (gdy MSb == 0) następuje jedynie przesunięcie w lewo (`crc <<= 1`).
- `xorout` wynosi `0x0000`, więc na wyjściu zwracana jest  zawartość 16-bitowego rejestru bez odwracania bitów.

CRC-16/CCITT-FALSE a AX.25 FCS
- **Różnice** CRC-16/CCITT-FALSE przetwarza bity MSB-first (poly `0x1021`, init `0xFFFF`, xorout `0x0000`). AX.25 FCS korzysta z LSB-first, używając odwróconego wielomianu `0x8408` oraz xorout `0xFFFF`, czyli rejestr na wyjściu jest negowany.
- Problem polega na tym ze dla uzycia algorytmu z AX.25 pakiet radiowy zostałby odrzucony przez odbiornik jako uszkodzony.

- **Testy** 
`test_crc_empty_input`: Pusty bufor wejściowy `b""` nie wykonuje pętli i zwraca initial value `0xFFFF`.
- `test_crc_standard_vector_123456789`: ten ciąg testowy dla CRC-16 zwraca `0x29B1`.
- `test_crc_not_ax25_fcs`: czy wynik dla `"123456789"` jest rozny od `0x906E`? (poprawny wynik dla AX.25).
- `test_crc_single_zero_byte` i `test_crc_single_ff_byte`: Sprawdzają zachowanie rejestru przy skrajnych wartościach bitowych (`0x00` -> `0xE1F0`, `0xFF` -> `0xFF00`).
- `test_crc_128_bytes_sequence`: Testuje ciąg 128 bajtów (`0x00..=0x7F`, wynik `0x1800`), czyli maksymalny rozmiar ładunku w ramce radiowej.

## 3.`define RadioPacket and PacketError types`
- RadioPacket - zawiera payload (`Vec<u8>`) oraz sumę kontrolną (`u16`). Struktura przedstawia TYLKO poprawny, sprawdzony pakiet w pamięci.
- PacketError - Zbiór błędów jako enum. Każdy wariant ma dołączony kontekst jako liczbę (`len`, `declared`, `actual`, `expected`, `count`), co moze ułatwić diagnostykę problemów.
- Funkcje parsera i budowy ramki nie wywołują `panic`, tylko zwracają `Result<T, PacketError>`.
- Wprowadzono traity `Display` oraz `std::error::Error`, przez co `PacketError` łączy się z biblioteką standardową `std`.

## 4. `implement packet building with tests`
#### Budowa ramki radiowej
| preamble | sync | length | payload | crc |
| --- | --- | --- | --- | --- |
| 5 x 0xAA | 0x7E | 1 byte | 0..=128 bytes | 2 bytes |

Uwagi co do CRC:
- jest w formie Big-Endian (`crc.to_be_bytes()`), 
- liczone TYLKO z 

#### Wprowadzone zmiany
- Prealokuję pamięć, wykorzystując to, że ramka ma ustalone wymiary (`Vec::with_capacity(5 + 1 + 1 + payload.len() + 2)`).
- Spełnienie warunku `payload.len() > 128` na początku funkcji zwraca `PacketError::PayloadTooLong`.
- Do sumy CRC wchodzi bajt zadeklarowanej długości oraz właściwy ładunek (`[length] + payload`). Preambuła i sync nie wchodzą do sumy CRC, bo ich zadaniem jest powiadomienie odbiornika, w którym momencie szumu danych zaczyna się pakiet.




