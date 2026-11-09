# NOTES: r2-radio-packet

# CRC-16 CCITT-FALSE
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
- liczone TYLKO z [length] + payload

#### Wprowadzone zmiany
- Prealokuję pamięć, wykorzystując to, że ramka ma ustalone wymiary (`Vec::with_capacity(5 + 1 + 1 + payload.len() + 2)`).
- Spełnienie warunku `payload.len() > 128` na początku funkcji zwraca `PacketError::PayloadTooLong`.
- Do sumy CRC wchodzi bajt zadeklarowanej długości oraz właściwy ładunek (`length + payload`). Preambuła i sync nie wchodzą do sumy CRC, bo ich zadaniem jest powiadomienie odbiornika, w którym momencie szumu danych zaczyna się pakiet.

## 5. `implement parse method type checking and tests`
Kolejność sprawdzania w `parse` ma znaczenie dla stabilności i bezpieczeństwa:
1. `data.len() < 9` (`TooShort`) eliminuje błędy wyjścia poza zakres (`panic: index out of bounds`) przy odczycie nagłówka i CRC.
2. `BadPreamble` oraz `BadSync` wcześnie odrzuca losowy szum przed dokładniejszą analizą pakieti.
3. `PayloadTooLong` (`declared > 128`): zabezpieczenie przed atakami przez sprawdzenie zadeklarowanego rozmiaru zanim zaufamy danym z nagłówka.
4. `LengthMismatch` oraz `TrailingBytes` oceniają zgodność rozmiaru bufora bez uruchamiania obliczeń sumy kontrolnej.
5. `CrcMismatch`: weryfikacja spójności danych, wykrywa uszkodzenia danych oraz nieprawidłową kolejność bajtów (np. Little-Endian).

## 6. `finalized crate documentation, README.md and NOTES.md`
- Re-eksporty w lib.rs. Zastosowano (`pub use packet::{...}`), co udostępnia najważniejsze API bezpośrednio z korzenia biblioteki, zachowując podział na `crc` i `packet`.
- Kod został zweryfikowany za pomocą `clippy` z flagą `-D warnings`.
- Dodano krótką dokumentację w `lib.rs`.

# Branch AX.25 FCS
Od tej pory każdy commit będzie poprawiał poprzednią realizację projektu tak, aby zmienić algorytm CRC z CCITT-FALSE na AX.25 FCS. Dotychczasowy kod z poprzednim algorytmem będzie zakomentowany jeśli wywołuje błędy, stopniowo wprowadzając poprawki.

## 7. `redesign of the frame and CRC algorithm with crc_vectors tests`
### Zmiana algorytmu z CRC-16/CCITT-FALSE na HDLC/AX.25 FCS:
Tablica zgodna z RFC 1549 Appendix A / Dire Wolf
- HDLC (High-Level Data Link Control) standard radiowy na którym bazuje m.in. AX.25
- RFC (Request for Comments) dokumentacja
- FCS (Frame Check Sequence) to suma kontrolna umieszczana na końcu ramki (dla AX.25 16-bitowe CRC)
| Parameter | Value |
| --- | --- |
| Polynomial | `0x8408` (odwrócone `0x1021`) |
| Initial value | `0xFFFF` |
| Reflect input | yes (LSB-first) |
| Reflect output | yes |
| XOR out | `0xFFFF` |
| Check (`"123456789"`) | `0x906E` |
| Bit order | LSB-first |

#### Żeby nie pomieszać kolejności:
Kategoria |	Za co odpowiada  |	Używane pojęcia |	Gdzie występuje
| --- | --- | --- | --- |
| Kolejność bitów | kolejność bitów wewnątrz jednego bajtu w obliczaniu CRC |	LSB-first, MSB-first, RefIn, RefOut, reflected poly 0x8408  | crc16_ax25 |
|Poziom bajtów | w jakiej kolejności 2 gotowe bajty są obok siebie |	Big-Endian (to_be_bytes), Little-Endian (to_le_bytes) |	 funkcja build() i parse()

### Zawartość testu wektorów referencyjnych z kodem Dire Wolf:

```c
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include "fcs_calc.h"

int main(void) {
    printf(" Dire Wolf data verification with fcs_calc \n\n");

    // reference vectors
    printf("\"\"                 -> 0x%04X\n", fcs_calc((unsigned char*)"", 0));
    printf("\"123456789\"        -> 0x%04X\n", fcs_calc((unsigned char*)"123456789", 9));
    printf("\"A\"                -> 0x%04X\n", fcs_calc((unsigned char*)"A", 1));

    unsigned char b00 = 0x00;
    unsigned char bff = 0xFF;
    printf("0x00               -> 0x%04X\n", fcs_calc(&b00, 1));
    printf("0xFF               -> 0x%04X\n", fcs_calc(&bff, 1));

    // 128-byte sequence
    unsigned char seq[128];
    for (int i = 0; i < 128; i++) seq[i] = (unsigned char)i;
    printf("0x00..=0x7F [128B] -> 0x%04X (dec: %u)\n", fcs_calc(seq, 128), fcs_calc(seq, 128));
    printf("0x00..0x7F  [127B] -> 0x%04X (dec: %u)\n", fcs_calc(seq, 127), fcs_calc(seq, 127));

    // packets ([length] + payload)
    unsigned char pkt_empty[] = { 0x00 };
    printf("Pkt empty ([0x00]) -> 0x%04X\n", fcs_calc(pkt_empty, 1));

    unsigned char pkt_ping[] = { 0x04, 'p', 'i', 'n', 'g' };
    printf("Pkt \"ping\"        -> 0x%04X\n", fcs_calc(pkt_ping, 5));

    unsigned char pkt_128zeros[129];
    pkt_128zeros[0] = 0x80;
    memset(&pkt_128zeros[1], 0x00, 128);
    printf("Pkt 128 zeros      -> 0x%04X\n", fcs_calc(pkt_128zeros, 129));

    return 0;
}
```
Wyniki testu Dire Wolf:

```sh
kornel@Mac ~ % cd ~/dev/direwolf-official && clang -O2 test_wektory.c src/fcs_calc.c -Isrc -o test_wektory && ./test_wektory
```
```
 Dire Wolf data verification with fcs_calc


""                 -> 0x0000
"123456789"        -> 0x906E
"A"                -> 0xA3F5
0x00               -> 0xF078
0xFF               -> 0xFF00
0x00..=0x7F [128B] -> 0xF1B9 (dec: 61881)
0x00..0x7F  [127B] -> 0xC96E (dec: 51566)
Pkt empty ([0x00]) -> 0xF078
Pkt "ping"        -> 0xC98D
Pkt 128 zeros      -> 0xD834
```

Wykorzystanie faktu, że operacja liczenia sumy kontrolnej jest liniowa: $CRC(A \oplus B) = CRC(A) \oplus CRC(B)$ pozwala na poniższą optymalizację (działanie bajt po bajcie z tabelą, zamiast bit po bicie).

### Na czym polega zoptymalizowanie z działania bit po bicie na przejście do wersji tabelarycznej?
Pętla wygląda tak, że zaczynam z init_value na CRC, czyli `0xFFFF`. I pierwszy krok: przesuwam o bajt w prawo (lewa strona XOR'a). Drugi krok: biorę początkową wartość, `0xFFFF`, liczę XOR z bajtem payload, sprawdzam jaki jest wynik, oddzielam tak, żeby było do 255. Trzeci krok: poprzedni wynik wrzucam do tabeli, która mówi jaki jest wynik dla złożonego 8-krotnego przejścia przesunięć i nakładania (bądź nie) wielomianu `0x8408` dla wyliczonego bajtu indeksu. Czwarty krok: nowy przesunięty o bajt CRC=`0xFFFF` liczę XOR z wartością pobraną z tablicy -> otrzymuję nowy CRC. Pętla powtarza się dla nowego CRC i bierze kolejny bajt z payload. Po wyjściu z pętli XOR-out, czyli XOR wartości z 4. kroku i `0xFFFF`, czyli zanegowanie każdego bitu.

## 8. `update packet build vectors for AX.25 FCS`
Poprawiona budowa sprawdzanych pakietów aby zapewnić zgodność z algorytmem AX.25 FCS.
Wytyczne:
- "" → aaaaaaaaaa7e00f078,
- "ping" → aaaaaaaaaa7e0470696e67c98d,
- "123456789" → aaaaaaaaaa7e0931323334353637383907ff,
- 128 zer → aaaaaaaaaa7e80 + 128×00 + d834

Dla testu ramki z payload = 128 x 0x00 (bajt length 0x80 + 128 bajtów 0x00) kod zwracał sumę `0x100E` podczas gdy wytyczne oraz sprawdzenie dla kodu z aplikacji Dire Wolf (`fcs_calc.c`) podawały `0xD834` (kod testu powyżej). Weryfikacja algorytmu pokazała błąd w tablicy `FCSTAB`. Pod indeksem 79 znajdowała się wartość `0xba53` zamiast poprawnej `0xbaf3`. Po poprawieniu literówki wyniki testów `test_crc_128_bytes_sequence` (`crc_vectors.rs`) oraz `test_build_128_zeros_payload` (`packet_vectors.rs`) zgadzają się z `fcs_calc.c`.

## 9. `enabled packet error handling tests`
Odkomentowałem testy błędów podczas tworzenia pakietów np. błędnej preambuły, długości, pomieszanego CRC i tak dalej. Były niezależne od budowy algorytmu, więc nie wymagały zmian.

## 10. `README updates according with AX.25 FCS and other minor changes`
uwaga: Pominąłem testowanie pliku `r2_vectors.json` zawartego w `R2_radio_packet_crc.md`, jako że dotyczył algorytmu CRC-16 CCITT-FALSE.
- Nowy plik `QUESTIONS.md` z pytaniami ze specyfikacji zadania `R2_radio_packet_crc.md`
- poprawiona spójność wyjaśnień w `NOTES.md`
- zaktualizowane `README.md`:
    - dodany opis tablicy `FCSTAB`
    - wyjaśniony algorytm AX.25 FCS wraz z przykładem
    - zmieniony opis struktury i szczegółów implementacyjnych algorytmu, wektorów testowych (referencyjnych)
- przykład kodu z `README.md` dodany do `main.rs`
- zaktualizowana krótka dokumentacja w `lib.rs`.

## 11. `finishing touches: added a git hook and fixed minor issues`
- poprawiono pozostawione cechy specyfikacji z poprzedniego algorytmu (`README.md`: MSB-First -> LSB-First, `crc_vectors.rs`: test returns `0xFFFF` -> `0x0000`)
- dodany skrypt hooka w ścieżce `githooks/pre-commit` (włączony przez `git config core.hooksPath githooks`).
- Skrypt automatyzuje weryfikację jakości kodu przed każdym commitem (`set -e`):
  1. `cargo fmt --all` z dołączaniem poprawionych plików (`git add -u`).
  2. `cargo clippy --all-targets --all-features -- -D warnings` blokuje commit przy wyrzuceniu jakiegokolwiek ostrzeżenia .
  3. `cargo test --quiet` potwierdza czy przechodzą testy, jak nie, commit zostanie odrzucony.
Żeby pominąć weryfikację przez `precommit` należy skorzystać z polecenia `git commit -n` (`--no-verify`).




