# Images from the web

This document names two images by URL, each on a different site. Letur fetches
neither until you press the button above the page.

![The image from the document that started this](https://cdn.prod.website-files.com/68a44d4040f98a4adf2207b6/6a8739a1b934ffe55bfc9715_44592f18.png)

The second is this project's own pipeline diagram, served from GitHub.

![How a document becomes a page](https://raw.githubusercontent.com/Ivapo/letur/main/tests/fixtures/samples/pipeline.svg)

## Things to try

1. Press **Fetch images from the web**. Both sites are allowed for this folder
   from now on, and both images arrive.
2. Change one letter of an image's file name, then stop typing. After a second
   it is fetched; a file the site does not have is refused, with **Try again**.
3. Put the letter back. The page returns from what was already fetched, and no
   request is made.
4. Open another document and come back to this one. It draws at once.
5. Quit Letur and open this file again. The sites are still allowed, so the page
   draws a second after it opens — with no press, and with one request per
   image, because the bytes are held in memory and never on disk.
