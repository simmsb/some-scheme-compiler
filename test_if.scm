(if 1
  (println "good")
  (println "bad"))

(if (- 1 1)
 (println "bad")
 (println "good"))

(if ((lambda () 1))
 (println "good")
 (println "bad"))

(if 0
 (println "bad"))

(if 1
 (println "good"))
