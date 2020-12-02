(define my-hash (ht-new))

(ht-set! my-hash 1 2)
(ht-set! my-hash 2 3)
(display (ht-get my-hash 1))
(display (ht-get my-hash 2))
