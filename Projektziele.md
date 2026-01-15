

hi claude! ich möchte ein projekt erstellen.

es soll folgende funktionen haben:

- DJ Lottery
  - DJs tragen sich mit alias und optional mit email in den lotterypool ein
  - zu festgelegter zeit wird die session gestartet, mit vordefinierter dauer (playtime)
  - das system lost die djs nacheinander aus, immer wenn einer anfängt, wird innerhalb weniger minuten der nächste gezogen (sonderfall: die erste losung, welche vor dem start der
  session passieren muss (sagen wir 20 minuten, aber das sollte anpassbar sein, z.b. in einer config))
  - später kommende djs können sich nachträglich eintragen und dem pool hinzufügen lassen
  - später kommende djs sollen eine niedrigere wahrscheinlichkeit haben gezogen zu werden (auch hier ggf. mit einer config oder ein modell)
- Das Erstellen eines timetables der einzelnen sets, wer wann gespielt hat soll im nachgang dazu dienen, den djs die aufnahmen automatisiert zuzusenden (bzw. einen upload in die
cloud und das zusenden eines links)
- das eintragen soll über eine website fungieren
- die einzelnen sessions sollen in einer datenbank gespeichert werden, so dass die datenbank der einzelnen sessions auch ausgewertet werden kann.
- djs die bei einer vorhergehenden session nicht gezogen wurden, sollen eine höhere wahrscheinlichhkeit bekommen, bei der aktuellen zu spielen (oder ggf. sogar garantierte
wahrscheinlichkeit ("joker")


ich wünsche einen modularen aufbau:
- gui (html, später ggf. auch mit externem webserver, so dass man sich dann nicht an einem lokalen rechner eintragen muss)
- statistikmodul der losung
- datenbankverwaltung (lesen, speichern), ggf. hier auch erstmala auf csv dateien mit timestamps (und ggf. ordner für die sets mit timestamp) und später die möglichkeit auf eine
datenbank umzustellen
- kommunikationsmodul zum zusenden der setlinks mit den hinterlegten emailadressen der registrierten djs
- storageverwaltung (zum upload der sets)
- kommunikation mit dem session recorder via API (siehe projekt sessionrecorder, /home/ffx/Projekte/session-recorder) um im nachgang die sets von der aufnahme zu beschaffen; im
besten fall später direkte verwaltung, ohne dass zwischengespeichert werden muss (direkte übergabe ans cloud modul und hochladen in die cloud)
- registrierungsmodul zum eintragen der DJS für eine session


sehr wichtig ist die möglichkeit alles zu erweitern.
