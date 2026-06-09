# Rust library to interface with the Bark notify app
By far my most complex project yet. It includes two API libraries (Gcal -> OAuth included, and bark notification API) implemented using my restman-rs library, a complex asynchronous iOS shortcuts web "interface" with sessions and internal redirects and a manager to top it all off. It's essentially a glorified todo list/time manager.  

# Next steps
- Figure out how to transfer state data in redirects (e.g., Homepage -> subpage)
- Figure out OneshotPages (do not generate a new session BUT the handler can only run b.cf ONCE)
- Probably more but I am brain dead righ tnow


- Test one-shot, multi-shot
    - [x] Firstly, is basic page functionality working?
        - I.e., all the ood actions (button, text input, timer, external url)  
    - [x] Can you access external state?
    - [x] ~~Is one-shot working?~~ Abandoned, don't need it + way too complex
    - 1) dynamic pages (e.g., pages with same handler but different URLs), 
    - 2) custom query parameters (e.g., page wiht custom handler), 
    - 3) static page
    - 4) pages only accessible through another page
    - 5) Test same page but parameter obtained differently (disconnect between OodSession and ParaHandler)
    - 6) test redirect cache persistence