# Corkboard

Build a simple RSS client for the terminal.
Unobtrusive, only show unread articles if prompted.
Not an article reader.

### Usage

Add a new RSS feed to the app.
> $corkboard add http://sampleblog.net/rss

Search for updates on all the feeds, show the new articles.
> $corkboard up
> <hash-1> http://sampleblog.net/todays-meal.html
> <hash-2> https://weirdsubscription.com/item-ac2
> <hash-3> https://sample.substack.com/p/fools-errand

Mark the article <hash-1> as read.
> $corkboard mark <hash-1>

Show all the articles that haven't been marked.
> $corkboard new
> https://weirdsubscription.com/item-ac2
> https://sample.substack.com/p/fools-errand

List all the feeds in the app
> $corkboard feeds
> http://sampleblog.net/rss
> https://weirdsubscription.com/rss
> https://sample.substack.com/feed

Remove a feed from the app
> $remove http://sampleblog.net/rss
