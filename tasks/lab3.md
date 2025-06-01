# Akademia Innowacyjnych Zastosowań Technologii Cyfrowych (AI Tech)

*projekt finansowany ze środków Programu Operacyjnego Polska Cyfrowa POPC 03.02.00-00-0001/20*

*Fundusze Europejskie Polska Cyfrowa, Rzeczpospolita Polska, Unia Europejska Europejski Fundusz Rozwoju Regionalnego*

---

## Zadanie 3. Wykorzystanie ocen ruchów z poprzednich iteracji i ruchów kandydackich w lokalnym przeszukiwaniu

Celem zadania jest poprawa efektywności czasowej lokalnego przeszukiwania w wersji stromej (steepest) z sąsiedztwem, które okazało się lepsze w poprzednim zadaniu. Jako rozwiązania startowe stosujemy rozwiązania losowe.

Stosujemy dwa mechanizmy poprawy efektywności:

1.  **Wykorzystanie ocen ruchów z poprzednich iteracji z uporządkowaną listą ruchów (LM - Lista Ruchów).** Na liście należy umieszczać ruchy zarówno między-, jak i wewnątrztrasowe. W przypadku ruchów wewnątrztrasowych wymiany dwóch krawędzi, należy dokładnie zapoznać się z opisem z wykładów dotyczącym problemu komiwojażera (TSP).
2.  **Ruchy kandydackie.**

Mechanizmy te stosujemy oddzielnie, czyli implementujemy dwie różne wersje lokalnego przeszukiwania. Opcjonalnie można zaimplementować trzecią wersję łączącą oba te mechanizmy.

### Przypomnienie z wykładu: Wykorzystanie ocen ruchów z poprzednich iteracji

**Algorytm lokalnego przeszukiwania z listą ruchów przynoszących poprawę:**

1.  Zainicjuj `LM` - listę ruchów przynoszących poprawę, uporządkowaną od najlepszego do najgorszego.
2.  Wygeneruj rozwiązanie startowe `x`.
3.  **powtarzaj**
4.      przejrzyj wszystkie nowe ruchy i dodaj do `LM` ruchy przynoszące poprawę.
5.      Przeglądaj ruchy `m` z `LM` od najlepszego do znalezienia aplikowalnego ruchu:
6.          Sprawdź czy `m` jest aplikowalny i jeżeli nie, usuń go z `LM`.
7.      **jeżeli** znaleziono aplikowalny ruch `m` **to**
8.          `x := m(x)` (zaakceptuj `m(x)`)
9.  **dopóki** nie znaleziono ruchu aplikowalnego `m` po przejrzeniu całej listy `LM`.

**Przykład dla TSP (wymiana dwóch krawędzi):**

* Oceniamy ruch związany z usunięciem krawędzi `(n1, succ(n1))` oraz `(n2, succ(n2))` i dodaniem krawędzi `(n1, n2)` oraz `(succ(n1), succ(n2))`.
* Zapamiętujemy ruch.
* *(Ilustracja - Obrazek 1 z oryginalnego PDF nie jest dołączony)*
    [Image 1]
* Po wykonaniu innego ruchu, zapamiętany ruch może stać się niepoprawny (np. jedna z krawędzi do usunięcia już nie istnieje), ale może też stać się poprawny później (np. po odwróceniu fragmentu trasy).

**Wnioski z przykładu:**

* Musimy brać pod uwagę kierunek przechodzenia krawędzi w bieżącym rozwiązaniu.
* Każdy inny ruch może tę kolejność zmienić.
* **3 sytuacje (kiedy przeglądamy ruchy `m` z `LM` od najlepszego):**
    1.  Usuwane krawędzie nie występują już w bieżącym rozwiązaniu -> usuwamy ruch `m` z `LM`.
    2.  Usuwane krawędzie występują w bieżącym rozwiązaniu w różnym od zapamiętanego kierunku -> zostawiamy ruch `m` w `LM`, ale nie aplikujemy, przechodzimy dalej.
    3.  Usuwane krawędzie występują w bieżącym rozwiązaniu w tym samym kierunku (także obie odwrócone) -> aplikujemy ruch `m` i usuwamy go z `LM`.
* Oceniając nowe ruchy, trzeba też uwzględniać (dodawać do `LM`) także ruchy dla odwróconego (względem obecnego) względnego kierunku krawędzi - nie są one aplikowalne do bieżącego rozwiązania, ale mogą się stać aplikowalne po wykonaniu innego ruchu.

**Wersja algorytmu dla TSP i wymiany dwóch krawędzi:**

1.  Zainicjuj `LM` - listę ruchów przynoszących poprawę, uporządkowaną od najlepszego do najgorszego.
2.  Wygeneruj rozwiązanie startowe `x`.
3.  **powtarzaj**
4.      przejrzyj wszystkie nowe ruchy (uwzględniając także ruchy z odwróconym względnym kierunkiem krawędzi) i dodaj do `LM` ruchy przynoszące poprawę.
5.      Przeglądaj ruchy `m` z `LM` od najlepszego do znalezienia aplikowalnego ruchu:
6.          **Jeżeli** co najmniej jednej z usuwanych krawędzi (zdefiniowanych przez ruch `m`) nie ma już w rozwiązaniu, usuń `m` z `LM`.
7.          **Jeżeli** obie usuwane krawędzie są w rozwiązaniu, ale w odwróconym względnym kierunku (względem zapamiętanego w `m`), pomiń `m`, ale pozostaw go w `LM`.
8.      **jeżeli** znaleziono aplikowalny ruch `m` **to**
9.          `x := m(x)` (zaakceptuj `m(x)`)
10. **dopóki** nie znaleziono aplikowalnego ruchu `m` po przejrzeniu całej listy `LM`.

### Ruchy kandydackie

* Jako kandydackie stosujemy ruchy wprowadzające do rozwiązania co najmniej jedną krawędź kandydacką.
* Krawędzie kandydackie definiujemy, wyznaczając dla każdego wierzchołka `k` (np. `k=10`) innych najbliższych wierzchołków. Parametr `k` można też dobrać eksperymentalnie.
* Przeglądanie ruchów (między trasami lub wewnątrz trasy) może wyglądać następująco:
    ```
    dla każdego wierzchołka n1:
        dla każdego wierzchołka n2 z listy k najbliższych wierzchołków n1:
            oceń ruch (ruchy) wprowadzający krawędź n1-n2 do rozwiązania
            (uwaga: nie jest to równoważne wymianie wierzchołka n1 z n2!)
    ```

### Opis eksperymentu

* Jako punkty odniesienia uruchamiamy też lokalne przeszukiwanie w wersji stromej bez powyższych mechanizmów oraz najlepszy algorytm z zadania 1 (heurystykę konstrukcyjną).
* Każdy z czterech algorytmów (LS stromy, LS z LM, LS z kandydatami, heurystyka konstr.) na każdej instancji uruchamiany 100 razy, startując (dla lokalnego przeszukiwania) z rozwiązań losowych.

---

## Sprawozdanie

* Analogiczne jak poprzednio (opis zadania, pseudokody, wyniki - wartości funkcji celu i czasy, wizualizacje, wnioski, kod).
