# Prezentace: Výroková logika

Samostatný HTML soubor `vyrokova-logika.html` — stačí ho otevřít v prohlížeči
(dvojklik), nic dalšího není potřeba. Žádné knihovny, žádné připojení k internetu:
i písma (Archivo, IBM Plex Sans a IBM Plex Mono, licence SIL OFL 1.1) jsou vložená
přímo v souboru, takže prezentace vypadá stejně i na školním počítači bez sítě.

## Ovládání

| klávesa | akce |
| --- | --- |
| `→`, `mezerník`, `PageDown` | další snímek |
| `←`, `PageUp` | předchozí snímek |
| `Home` / `End` | první / poslední snímek |
| `F` | celá obrazovka |
| `A` / `B` | přepnutí výroku A nebo B na interaktivním snímku |

Na telefonu a tabletu se listuje třemi způsoby: přejetím prstem, tlačítky `←` / `→`
v dolních rozích nebo ťuknutím na volnou plochu snímku (interaktivní části se tím
nespustí).

## Obsah (16 snímků)

1. Titulní snímek, přehled spojek a ovládání
2. Co je výrok
3. Přehled spojek + negace (interaktivní)
4. Konjunkce — definice a pravdivostní tabulka
5. **Konjunkce interaktivně** — sériové zapojení dvou vypínačů, animovaný proud v drátu
6. Disjunkce — definice a pravdivostní tabulka
7. **Disjunkce interaktivně** — paralelní zapojení dvou vypínačů
8. Implikace — definice a pravdivostní tabulka
9. **Implikace interaktivně** — „Pokud přineseš mléko, bude buchta.“
10. Implikace — další příklady a test „vadí nám to?“
11. Ekvivalence — definice a pravdivostní tabulka
12. **Ekvivalence interaktivně** — semafor a chodec na přechodu
13. Ekvivalence — další příklady a rozdíl mezi ⇒ a ⇔
14. Zápis, čtení nahlas a kdy co platí
15. Souhrnná tabulka všech spojek + de Morganovy zákony
16. Závěr

Poznámky pro toho, kdo prezentuje — historie oboru, co říct u jednotlivých listů
a další příklady — jsou v souboru [`poznamky-pro-prezentujiciho.md`](poznamky-pro-prezentujiciho.md).

Na každém interaktivním snímku se u výroků zobrazuje jejich pravdivostní hodnota
(0/1), v tabulce se zvýrazňuje aktuální řádek a pod ní je slovní vyhodnocení.

## Vzhled

Prezentace je vysázená jako technický výkres: světlý papír s milimetrovou sítí,
rámeček listu a popisové pole dole. Schémata používají skutečné referenční
označení součástek (GB1 baterie, S1/S2 vypínače, HL1 žárovka) a u konjunkce
a disjunkce je i značka logického hradla podle ČSN EN 60617-12. Jantarová je
v celé prezentaci jediná výrazná barva a znamená vždy totéž — „teče proud“,
tedy pravdivostní hodnotu 1.
