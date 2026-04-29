PERFORMER "Test Artist"
TITLE "Malformed Album"
FILE "album.flac" WAVE

  TRACK 01 AUDIO
    TITLE "Valid Track"
    PERFORMER "Test Artist"
    INDEX 01 00:00:00

  TRACK 02 AUDIO
    TITLE "This track has a bad INDEX format"
    PERFORMER "Test Artist"
    INDEX 01 NOT_A_VALID_TIME

  TRACK 03 AUDIO
    THIS IS NOT A VALID CUE COMMAND
    TITLE "After Error"
    INDEX 01 06:00:00

  TRACK 04 AUDIO
    TITLE "Incomplete"
    PERFORMER "Test Artist"
