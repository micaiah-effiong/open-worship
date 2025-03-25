**Up Next:**

- Refactor and clean up parser code
- Add dev installation instructions for different OS
- Scale user defined font size on screen resize
  In activity_screen allow user to define font size, and scale it appropriatly
  on the display screen and adapt to resizing.
  Also prevent sizes over MAX_FONT_VALUE for the screen.
- Preserve previously selected translation
  When downloading bible and reloading translations, ensure previously
  selected translation is not altered

---

** Bug **

- Switching bible translation is broken
  If you have multiple translation(versions) of the bible, when trying to switch
  to another, shows texts from a different version
- Songs without verses should not be added
  When adding song to database, it must have a minimum of 1 verse
- Vertical text break auto max font adjust
