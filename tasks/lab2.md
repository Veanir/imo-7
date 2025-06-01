# Akademia Innowacyjnych Zastosowań Technologii Cyfrowych (AI Tech)

*projekt finansowany ze środków Programu Operacyjnego Polska Cyfrowa POPC 03.02.00-00-0001/20*

*Fundusze Europejskie Polska Cyfrowa, Rzeczpospolita Polska, Unia Europejska Europejski Fundusz Rozwoju Regionalnego*

---

## Zadanie 2. Lokalne przeszukiwanie

Zadanie polega na implementacji lokalnego przeszukiwania w wersjach stromej (steepest) i zachłannej (greedy), z dwoma różnym rodzajami sąsiedztwa, startując albo z rozwiązań losowych, albo z rozwiązań uzyskanych za pomocą jednej z heurystyk opracowanych w ramach poprzedniego zadania. W sumie 8 kombinacji - wersji lokalnego przeszukiwania.

Jako punkt odniesienia należy zaimplementować algorytm losowego błądzenia, który w każdej iteracji wykonuje losowo wybrany ruch (niezależnie od jego oceny) i zwraca najlepsze znalezione w ten sposób rozwiązanie. Algorytm ten powinien działać w takim samym czasie jak średnio najwolniejsza z wersji lokalnego przeszukiwania.

### Sąsiedztwa

W przypadku rozważanego problemu potrzebne będą dwa typy ruchów: ruchy zmieniające zbiory wierzchołków tworzące dwa cykle i ruchy wewnątrztrasowe, które jedynie zmieniają kolejność wierzchołków na trasie.

Jako ruch zmieniający zbiór wierzchołków wykorzystujemy wymianę dwóch wierzchołków pomiędzy dwoma cyklami.

Stosujemy dwa rodzaje ruchów wewnątrztrasowych (jeden albo drugi, stąd dwa rodzaje sąsiedztwa):
1.  Wymiana dwóch wierzchołków wchodzących w skład trasy.
2.  Wymiana dwóch krawędzi.

*(Ilustracje ruchów - Obrazki 1-4 z oryginalnego PDF nie są dołączone)*
[Image 1]
[Image 2]
[Image 3]
[Image 4]

Implementacja musi wykorzystywać obliczanie delty funkcji celu.

Sąsiedztwo składa się więc z ruchów dwóch typów. W wersji stromej przeglądamy wszystkie ruchy obu typów i wybieramy najlepszy. W wersji zachłannej należy zrandomizować (kolejność nie musi być całkowicie losowa) kolejność przeglądania. W sprawozdaniu proszę opisać sposób randomizacji.

Każdy algorytm na każdej instancji uruchamiany 100 razy startując z rozwiązań losowych lub rozwiązań uzyskanych za pomocą jednej (najlepszej) z heurystyk opracowanych w poprzednim zadaniu.

### Lokalne przeszukiwanie w wersji zachłannej (greedy):

1.  Wygeneruj rozwiązanie startowe `x`
2.  **Powtarzaj**
3.      **dla każdego** `m ∈ M(x)` w losowej kolejności
4.          **jeżeli** `f(m(x)) > f(x)` **to**
5.              `x := y` (*Uwaga: w oryginale jest `x:=y`, co może być błędem; prawdopodobnie powinno być `x := m(x)` lub podobnie, oznaczając akceptację ruchu*)
6.  **dopóki** nie znaleziono lepszego rozwiązania po przejrzeniu całego `N(x)`

### Lokalne przeszukiwanie w wersji stromej (steepest):

1.  Wygeneruj rozwiązanie startowe `x`
2.  **powtarzaj**
3.      znajdź najlepszy ruch `m ∈ M(x)`
4.      **jeżeli** `f(m(x)) > f(x)` **to** (*Uwaga: w oryginale jest `i f(m(x)) > f(x)`, 'i' może być błędem typograficznym*)
5.          `x := m(x)`
6.  **dopóki** nie znaleziono lepszego rozwiązania po przejrzeniu całego `M(x)`

---

## Sprawozdanie

W sprawozdaniu należy umieścić:

* Krótki opis zadania.
* Opis wszystkich zaimplementowanych algorytmów w pseudokodzie.
* Wyniki eksperymentu obliczeniowego. Dla każdej kombinacji instancja/algorytm należy podać wartość średnią, minimalną i maksymalną funkcji celu oraz, w odrębnej tabeli, analogiczne dane dla czasu obliczeń. Sugerowany format tabeli jak poprzednio.
* Wizualizacje najlepszych rozwiązań dla każdej kombinacji podobnie jak poprzednio.
* Wnioski.
* Kod programu (np. w postaci linku).
