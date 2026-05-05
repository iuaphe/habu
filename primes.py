class Loop:
    continue_flag = False
    semaphore = 0


i = 0
for k in range(-(2//-1), int(100)+1):
    for f in range(-(2//-1), int(k ** 0.5)+1):
        if k % f == 0:
            Loop.semaphore = 1
            Loop.continue_flag = True
            break
    if Loop.semaphore > 0:
        Loop.semaphore -= 1
        if Loop.semaphore == 0 and Loop.continue_flag:
            Loop.continue_flag = False
            continue
        break
    i = i + 1
    print(k)
# semaphore thing omitted because base
