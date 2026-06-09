- [x] Need to be able to insert new events (transit events)
- [x] Mutate existing events 
- [x] Read existing events

# Tags
Everything on the schedule calendar is parsed and user is notified about errors
- Do this nightly; review calendar and tell the backend to parse - it will report errors => fix until everything is good

Possible tags
- Priority: `%p1` - priority 1 (highest), `%p2` (next lowest), `%pl` (lowest), `ph` (immutable)
- When you add a new event that overlaps with one you already had, the old one is made red and app tries shifting future events


<tags>
p1
</tags>