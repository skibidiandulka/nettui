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

## Obsah (6 snímků)

Na plátně je záměrně jen to, co má smysl ukazovat — výklad, definice a pravdivostní
tabulka jdou na tabuli.

1. Titulní snímek s přehledem znaků
2. **Konjunkce** — sériové zapojení dvou vypínačů, animovaný proud v drátu
3. **Disjunkce** — paralelní zapojení, proud teče jen sepnutou větví
4. **Implikace** — „Pokud přineseš mléko, bude buchta.“
5. **Ekvivalence** — semafor a chodec na přechodu
6. Závěr

Na každém snímku se u výroků zobrazuje jejich pravdivostní hodnota (0/1) a pod
schématem je jednořádkové vyhodnocení.

## Podklad pro výklad

[`podklad-pro-vyklad.html`](podklad-pro-vyklad.html) je souvislý text pro toho, kdo
prezentuje — od definice výroku přes formuli, pravdivostní hodnotu a negaci ke spojkám,
krátká historie a pak průchod všemi čtyřmi ukázkami. Značky <b>Tabule</b> a <b>Plátno</b>
říkají, co se kdy kreslí a co se ukazuje.

Je to jeden průběžně čtený dokument, ne prezentace: černá na bílé, sazba písmem Literata
(navrženým pro čtečky), žádný JavaScript ani animace — určeno ke čtení na e-ink tabletu
během výkladu.

## Vzhled

Prezentace je vysázená jako technický výkres: světlý papír s milimetrovou sítí,
rámeček listu a popisové pole dole. Schémata používají skutečné referenční
označení součástek (GB1 baterie, S1/S2 vypínače, HL1 žárovka) a u konjunkce
a disjunkce je i značka logického hradla podle ČSN EN 60617-12. Jantarová je
v celé prezentaci jediná výrazná barva a znamená vždy totéž — „teče proud“,
tedy pravdivostní hodnotu 1.
