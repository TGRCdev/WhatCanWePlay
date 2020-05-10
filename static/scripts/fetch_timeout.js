function timeout(promise, timeout) {
    timeout_promise = new Promise((resolve, reject) => {
        setTimeout(reject, timeout, 'timeout')
    })
    return Promise.race([
        promise, timeout_promise
    ])
}