# Poznámky pro prezentujícího

Stručné zápisky k prezentaci `vyrokova-logika.html`. Nic z toho se nemusí říct doslova —
je to opora, ne scénář.

**Ovládání:** `→` nebo mezerník dál, `←` zpět, `F` celá obrazovka, `A` / `B` přepínají
výroky na interaktivních snímcích. Na telefonu se listuje přejetím prstem, tlačítky
v rozích nebo ťuknutím na volnou plochu snímku.

---

## 1. Odkud se výroková logika vzala (~2 minuty)

Celé to drží pohromadě jedna linka: **od řazení pojmů k řazení celých vět a od nich
k drátům v počítači.**

**Aristotelés** (384–322 př. n. l.) — první, kdo logiku sepsal jako systém. Zajímá ho
*vnitřek* věty: rozděluje pojmy do kategorií a skládá sylogismy typu „Všichni lidé jsou
smrtelní. Sókratés je člověk. Tedy je smrtelný.“ Pracuje s tím, co je ve větě uvnitř —
kdo je podmět, kdo přísudek, jestli platí „všichni“ nebo „někteří“.

**Chrysippos ze Soloi** (asi 280–206 př. n. l.) — stoik, pocházel ze Soloi na **Kypru**
a vedl stoickou školu v Athénách. Udělal krok, na kterém stojí celá naše prezentace:
přestal rozebírat vnitřek věty a začal brát **celý výrok jako jeden stavební kámen**,
který se s dalšími spojuje slovy *a*, *nebo*, *jestliže — pak*. Sestavil pět základních
úsudkových schémat, mezi nimi to nejznámější: „Je-li první, pak druhé. Je první. Tedy je
druhé.“ To je přesně naše implikace. Ve starověku se o něm říkalo: *„Kdyby bohové měli
logiku, byla by to logika Chrysippova.“* Napsal přes sedm set spisů a nedochoval se ani
jediný celý — známe ho hlavně z citací u jiných autorů.

**Filón z Megary** (o něco starší současník) — jako první definoval implikaci úplně
stejně, jak ji učíme dnes: *nepravdivá je jedině tehdy, když předpoklad platí a závěr ne.*
Už tehdy se o to vedly spory, protože to odporuje běžné řeči. Dobrá poznámka, když se
někdo ozve, že „když 2 + 2 = 5, jsem papež“ je divné — divné to připadalo lidem už před
dvěma tisíci lety.

**Pak dlouhá pauza.** Ve středověku se logika hlavně komentovala. V 17. století
**Leibniz** vysloví myšlenku, že by se spory daly řešit výpočtem — „posaďme se a
počítejme“ — ale nástroje na to ještě nejsou.

**George Boole** (1854, *Zákony myšlení*) — nástroj dodá. Ukáže, že se s výroky dá počítat
jako s čísly, když připustíme jen dvě hodnoty: **1 a 0**. Logika se stává algebrou.

**Claude Shannon** (1937) — ve 21 letech v magisterské práci na MIT spojí obě větve
dohromady: Booleova algebra přesně popisuje **spínací a reléové obvody**. Sériové zapojení
se chová jako konjunkce, paralelní jako disjunkce. Bývá označována za jednu z nejvlivnějších
diplomových prací vůbec — vede od ní přímá cesta k logickým hradlům a procesorům.

**Pointa na závěr historie:** proto v téhle prezentaci klikáme na vypínače. Není to
berlička pro názornost — je to doslova to, co Shannon objevil.

---

## 2. Co říct u jednotlivých listů

| List | O čem mluvit |
| --- | --- |
| 1 | Titulní. Zmínit, že příklady se dají naklikat a že je na nich postavený výklad. |
| 2 | Výrok = věta, u které má smysl ptát se, jestli je pravdivá. Otázka ani rozkaz výrok není. |
| 3 | Přehled spojek — jen letmo, každá přijde na řadu zvlášť. Negaci si nech přepnout od někoho z publika. |
| 4 | Konjunkce: pravdivá jen v jediném případě. Ukázat na tabulce tu jedinou jedničku. |
| 5 | **Interaktivní.** Nech někoho rozepnout vypínač. Pointa: stačí jedna nula a je po všem. |
| 6 | Disjunkce: pozor, „nebo“ v logice není vylučovací — obě možnosti smí platit zároveň. |
| 7 | **Interaktivní.** Proud si najde cestu. Jediná nula v tabulce = obě větve rozepnuté. |
| 8 | Implikace: předpoklad a závěr. Zdůraznit, že neříká nic o příčině. |
| 9 | **Interaktivní.** Projít všechny čtyři kombinace mléka a buchty. |
| 10 | Test „vadí nám to?“ — nejdůležitější list pro pochopení implikace, nespěchat. |
| 11 | Ekvivalence = implikace v obou směrech, „nutná a postačující podmínka“. |
| 12 | **Interaktivní.** Chodec na červenou: hodnoty se liší, výrok je nepravdivý. |
| 13 | Test „jde jedno bez druhého?“ a rozdíl mezi ⇒ a ⇔ (mokrý chodník). |
| 14 | Tabulka zápisu a čtení — vhodné místo pro fotku od publika. |
| 15 | Souhrnná tabulka všech šestnácti hodnot. |
| 16 | Závěr. |

---

## 3. Implikace: jak o ní mluvit

Implikace je **slib**, ne příčina. Ptáme se jedinou otázkou: *Vadí nám to? Byl slib porušen?*
Porušit ho jde jediným způsobem — **předpoklad platí a závěr ne.** Všechno ostatní je pravda.

- „Když přineseš mléko, bude buchta.“ Mléko nebylo, buchta je. **Vadí nám, že je buchta?**
  Ne — nikdo neslíbil, že bez mléka buchta nebude. → **1**
- Mléko bylo, buchta není. → slib porušen → **0**. Jediná nula v celé tabulce.
- „Když bude svítit slunce, půjdeme na výlet.“ Bylo zataženo a šli jsme stejně — o zataženém
  dni slib nic neříkal. → **1**
- „Je-li číslo dělitelné šesti, je dělitelné třemi.“ Protipříklad nenajdeš → platí vždy. → **1**
- „Když 2 + 2 = 5, jsem papež.“ Předpoklad nikdy nenastane, slib nejde porušit. → **1**
  (Z nepravdy plyne cokoli.)

**Časté nedorozumění:** studenti chtějí u A = 0 odpovědět „nevím“. Odpověď zní: logika nemá
třetí hodnotu, a když se nedá nic vytknout, bereme to jako pravdu.

---

## 4. Ekvivalence: jak o ní mluvit

Otázka zní: *Jdou obě věty vždycky ruku v ruce?* Stačí jeden případ, kdy platí jedna a druhá
ne, a ekvivalence padá.

- „Žárovka svítí právě tehdy, když je vypínač sepnutý.“ → **1**
- „Číslo je sudé právě tehdy, když končí 0, 2, 4, 6 nebo 8.“ Platí oběma směry — proto je to
  definice. → **1**
- „Prší právě tehdy, když je mokrý chodník.“ Chodník může být mokrý od kropicího vozu →
  **0**. Ale „prší ⇒ mokrý chodník“ pořád platí. **Nejlepší příklad na rozdíl ⇒ a ⇔.**
- „Zkoušku uděláš právě tehdy, když získáš aspoň 60 bodů.“ Body jsou podmínka nutná
  i postačující. → **1**

Pozor na ten nenápadný případ: **červená a chodec stojí** je taky pravda. Obě hodnoty jsou
nula, tedy se shodují.

---

## 5. Zápis a čtení

| Spojka | Znak | Čteme nahlas | Pravdivá, když | Nepravdivá, když |
| --- | --- | --- | --- | --- |
| Negace | ¬A | „ne A“ | A neplatí | A platí |
| Konjunkce | A ∧ B | „A a zároveň B“ | platí obojí | aspoň jedno neplatí |
| Disjunkce | A ∨ B | „A nebo B“ | platí aspoň jedno | neplatí ani jedno |
| Implikace | A ⇒ B | „jestliže A, pak B“ | ve všech ostatních případech | A platí a B neplatí |
| Ekvivalence | A ⇔ B | „A právě tehdy, když B“ | hodnoty se shodují | hodnoty se liší |

Implikaci lze číst i jako „z A plyne B“ nebo „A je postačující podmínka pro B“.
U ekvivalence se říká „nutná a postačující podmínka“.

## 6. Všechny kombinace

| A | B | slovy | A ∧ B | A ∨ B | A ⇒ B | A ⇔ B |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 0 | nepravda, nepravda | 0 | 0 | 1 | 1 |
| 0 | 1 | nepravda, pravda | 0 | 1 | 1 | 0 |
| 1 | 0 | pravda, nepravda | 0 | 1 | 0 | 0 |
| 1 | 1 | pravda, pravda | 1 | 1 | 1 | 1 |

Zapamatovatelné zkratky: **∧** má jedinou jedničku, **∨** jedinou nulu, **⇒** má nulu jen
u „1 ⇒ 0“, **⇔** má jedničku tam, kde se hodnoty shodují.

---

## 7. Otázky do publika

- Je „Zavři okno!“ výrok? (Ne — rozkaz.)
- Platí „Když bude pršet, vezmu si deštník“, když neprší a deštník si vezmu? (Ano.)
- Proč je sériové zapojení konjunkce a ne disjunkce?
- Kdy se liší „když — pak“ od „právě tehdy, když“? (Mokrý chodník.)
