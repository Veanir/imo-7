# Zadanie 4: Rozszerzenia lokalnego przeszukiwania

## Wymagania

Należy zaimplementować trzy metody:

* **(MSLS)** Multiple start local search - Lokalne przeszukiwanie z różnych losowych punktów startowych[cite: 1].
* **(ILS)** Iterated local search - Iteracyjne przeszukiwanie lokalne z niewielką perturbacją[cite: 1].
* **(LNS)** Large neighborhood search - z większą perturbacją typu Destroy-Repair[cite: 1].

Jako lokalne przeszukiwanie stosujemy najlepszą metodę z poprzednich zajęć[cite: 1].

## Perturbacje

W obu wypadkach (ILS/LNS) szczegółowe perturbacje należy zaproponować samodzielnie i opisać je w sprawozdaniu. Należy dążyć do tego, aby metody ILS/LNS dawały wyniki lepsze niż MSLS[cite: 1].

* **Perturbacja 1 (ILS):** Może polegać np. na wymianie kilku krawędzi i/lub wierzchołków na inne wybrane losowo[cite: 1, 2].
* **Perturbacja typu Destroy-Repair (LNS):** Powinna polegać na usunięciu większej liczby krawędzi i wierzchołków (np. 30%) (destroy) i naprawieniu rozwiązania za pomocą metody heurystycznej, jednej z tych zaimplementowanych na pierwszych zajęciach[cite: 3]. Wierzchołki/krawędzie do usunięcia można wybierać losowo lub heurystycznie, np. te, które leżą blisko drugiego cyklu[cite: 4]. Tę wersję testujemy też bez lokalnego przeszukiwania (tylko początkowe rozwiązanie poddajmy lokalnemu przeszukiwaniu, o ile rozwiązanie startowe było losowe) (wersja LNSa)[cite: 5].

## Warunki Eksperymentu

* Każdą z metod uruchamiamy 10 razy[cite: 6].
* W ramach MSLS wykonujemy 200 iteracji lokalnego przeszukiwania[cite: 6]. Wynikiem końcowym MSLS jest najlepsze rozwiązanie uzyskane w tych 200 przebiegach[cite: 8].
* W przypadku $ILS/LNS$ warunkiem stopu jest osiągnięcie czasu równego średniemu czasowi MSLS dla tej samej instancji[cite: 7].
* We wszystkich wypadkach startujemy z rozwiązań losowych[cite: 9].
* Eksperymenty wykonujemy na instancjach `kroa200` i `krob200`[cite: 9].

## Sprawozdanie

Sprawozdanie - analogiczne jak poprzednio[cite: 9]. Należy dodać informacje o liczbie iteracji (perturbacji) w metodach $ILS/LNS$[cite: 10].

## Pseudokod Algorytmów

### Multiple start local search (MSLS)

```
Powtarzaj
  Wygeneruj zrandomizowane rozwiązanie startowe x
  Lokalne przeszukiwanie (x)
Do spełnienia warunków stopu

Zwróć najlepsze znalezione rozwiązanie
```
[cite: 10]

### Iterated local search (ILS)

```
Wygeneruj rozwiązanie początkowe x
x := Lokalne przeszukiwanie (x)
Powtarzaj
  y := x
  Perturbacja (y)
  y := Lokalne przeszukiwanie (y)
  Jeżeli f(y) > f(x) to
    x := y [cite: 11]
Do spełnienia warunków stopu
```
[cite: 10]

### Large neighborhood search (LNS)

```
Wygeneruj rozwiązanie początkowe x
x := Lokalne przeszukiwanie (x) (opcja)
Powtarzaj
  y := x
  Destroy (y)
  Repair (y)
  y := Lokalne przeszukiwanie (y) (opcja)
  Jeżeli f(y) > f(x) to
    x := y
Do spełnienia warunków stopu
```