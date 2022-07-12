#!/usr/bin/env bash

mkdir -p private-test-icals

cd private-test-icals

# check if bmi.ics exists
# if [ ! -f bmi.ics ]; then
#   curl "https://www.bmi.com/events/ical" > bmi.ics
# fi

# the bmi file is broken beyond repair

# check if americanhistory.ics exists
# if [ ! -f americanhistory.ics ]; then
curl "https://americanhistorycalendar.com/eventscalendar?format=ical&viewid=4" > americanhistory.ics
# fi

# https://calendar.google.com/calendar/ical/en.german%23holiday%40group.v.calendar.google.com/public/basic.ics
# check if german.ics exists
# if [ ! -f german.ics ]; then
curl "https://calendar.google.com/calendar/ical/en.german%23holiday%40group.v.calendar.google.com/public/basic.ics" > german.ics
# fi


# cargo run --release --example generate_random > generated.ics