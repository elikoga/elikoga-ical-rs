#!/usr/bin/env bash

mkdir -p private-test-icals

cd private-test-icals

# check if bmi.ics exists
if [ ! -f bmi.ics ]; then
  curl "https://www.bmi.com/events/ical" > bmi.ics
fi

# check if americanhistory.ics exists
if [ ! -f americanhistory.ics ]; then
  curl "https://americanhistorycalendar.com/eventscalendar?format=ical&viewid=4" > americanhistory.ics
fi

# cargo run --release --example generate_random > generated.ics