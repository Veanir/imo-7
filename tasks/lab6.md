Zadanie 6. Testy globalnej wypukłości.
Należy wygenerować dla każdej instancji 1000 losowych optimów lokalnych, tj. rozwiązań uzyskanych
z losowych rozwiązań startowych po zastosowaniu lokalnego przeszukiwania w wersji zachłannej.
Należy wygenerować także bardzo dobre rozwiązanie za pomocą najlepszej metody opracowanej do
tej pory. Następnie dla każdego z 1000 losowych lokalnych optimów należy policzyć podobieństwo do
bardzo dobrego rozwiązania i średnie podobieństwo dla wszystkich pozostałych optimów lokalnych z
tego zbioru. Na osi x nanosimy wartość funkcji celu, na y (średnie) podobieństwo. Liczymy też
wartość współczynnika korelacji.
Stosujemy dwie miary podobieństwa (oddzielnie):
- Liczba par wierzchołków przydzielonych w obu rozwiązaniach razem do jednego cyklu
- Liczba wspólnych krawędzi.