1. Show state of library
2. Show a naive rust implementation
3. What is missing?
4. Walk through

### Open Questions

1. Clarify noninteractive, adaptive branch  
    * filters and odometers are interactive
2. Why is there a stipulation that interactive compositors only accept interactive queries?
    * justification is to keep number of implementations down
3. Do all queryables that model interactive measurements need to talk to their parents?
    * yes, as conceivably all queryables need to be freezable 
4. Is it possible to have two IM's that both contend for the same resources?
    * not two IM's, but two odometers can be beneath a filter
