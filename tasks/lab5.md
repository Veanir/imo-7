Zadanie 5. Hybrydowy algorytm ewolucyjny
Cel
Celem jest implementacja hybrydowego algorytmu ewolucyjnego (HAE) oraz porównanie go z metodami MSLS, ILS i LNS zaimplementowanymi w poprzednim zadaniu.
Proponowane parametry algorytmu

Populacja elitarna o wielkości 20.
Algorytm steady state.
W populacji nie mogą znajdować się kopie tego samego rozwiązania (można porównywać całe rozwiązanie lub wartość funkcji celu).

Proponowany operator rekombinacji

Wybieramy jednego z rodziców jako rozwiązanie wyjściowe.
Usuwamy z tego rozwiązania wszystkie krawędzie, które nie występują w drugim rodzicu.
Usuwamy także wierzchołki, które stały się wolne (zostały usunięte obie sąsiednie krawędzie).
Rozwiązanie naprawiamy za pomocą metody heurystycznej, analogicznie jak w metodzie LNS.
Testujemy również wersję algorytmu bez lokalnego przeszukiwania po rekombinacji (nadal stosujemy lokalne przeszukiwanie dla populacji początkowej).

Modyfikacje w przypadku przedwczesnej zbieżności
Jeśli opisany powyżej algorytm powodowałby przedwczesną zbieżność, można go zmodyfikować, np.:

Wprowadzić dodatkowe mechanizmy dywersyfikacji populacji.
Usuwać więcej wierzchołków/krawędzi.

Można także opcjonalnie zaproponować inny własny operator rekombinacji.
Parametry eksperymentu
Parametry eksperymentu takie same jak w przypadku ILS/LNS.
Sprawozdanie

Analogiczne jak poprzednio.
Podajemy liczbę iteracji HAE oraz poprzednich metod.
Podajemy także wyniki heurystycznej metody zachłannej, tej samej, która została wykorzystana w metodach LNS i HAE.

Algorytm HAE z selekcją elitarną i steady state

Wygeneruj populację początkową X (stosując lokalne przeszukiwanie).
Powtarzaj:
Wylosuj dwa różne rozwiązania (rodziców) stosując rozkład równomierny.
Skonstruuj rozwiązanie potomne y poprzez rekombinację rodziców.
y := Lokalne przeszukiwanie (y) (opcjonalnie).
Jeżeli y jest lepsze od najgorszego rozwiązania w populacji i (wystarczająco) różne od wszystkich rozwiązań w populacji:
Dodaj y do populacji i usuń najgorsze rozwiązanie.




Dopóki nie są spełnione warunki stopu.

