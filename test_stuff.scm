(define go (lambda (i)
            (let ((ii (- i 1)))
             (if ii
                (let ()
                  (println ii)
                  (go ii)
                 )
              ))
            ))

(go 1000)
