# melonbooks_scraper
scrapes products from melonbooks from the given artists.
default notification system from melonbooks is bad, so I will do it myself.
in the future, other websites than melonbooks could be supported too.
requires a running db, tables will be created on the fly.
work in progress.

Usage: melonbooks-scraper <--daemon|--load-new|--refresh|--add-artist <ADD_ARTIST>|--remove-artist <REMOVE_ARTIST>>

Usage: melonbooks-scraper [OPTIONS]

Options:
  -d, --daemon                         run 'refresh' and 'load-new' in an 4h interval
  -l, --load-new                       scrape melonbooks for new products from stored artists
      --also-new-unavailable           use with 'load-new', scrape melonbooks for new products that are not available as well
  -r, --refresh                        scrape melonbooks for updates of local stored products
      --add-artist <ADD_ARTIST>        add artist to db, use 'load-new' afterwards to scrape products
      --remove-artist <REMOVE_ARTIST>  remove artist and their products from the db
      --site <SITE>                    required with 'add-artist' and 'remove-artist', specify from which site the products should be scraped from (only melonbooks supported for now)
  -h, --help                           Print help information

web interface not included in this project. (good luck)
