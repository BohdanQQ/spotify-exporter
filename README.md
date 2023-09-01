# spotify-exporter

Exports Spotify song data.

This tool exports Spotify song data into various formats. Currently exports 
`n` of your "top tracks" (as reported by the API). Outputs into Markdown, HTML, JSON, HTML-Markdown mix.

## Build

`cargo b --release`

## Usage

1. Create a Spotify API application
2. Create an `.env` file in the project root


    RSPOTIFY_CLIENT_ID=your_id
    RSPOTIFY_CLIENT_SECRET=your_secret
    RSPOTIFY_REDIRECT_URI=http://localhost/

3. run `spotify-exporter`, e.g.


    spotify-exporter -f json -o ./out-medium.json top-tracks medium -c 15

4. expect a browser to open, wait for redirect to `RSPOTIFY_REDIRECT_URI` 
5. paste the url into terminal
