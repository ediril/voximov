document.querySelectorAll('.waitlist-form').forEach((form) => {
  const row = form.querySelector('.waitlist-row')
  const submit = form.querySelector('button[type="submit"]')
  const status = form.querySelector('.waitlist-status')

  function show(outcome, message) {
    status.dataset.outcome = outcome
    status.textContent = message
    status.hidden = false
  }

  async function send() {
    try {
      const response = await fetch(form.action, {
        method: 'POST',
        headers: { Accept: 'application/json' },
        body: new FormData(form),
      })
      return (await response.json()).success === true
    } catch {
      return false
    }
  }

  form.addEventListener('submit', async (event) => {
    event.preventDefault()
    submit.disabled = true
    status.hidden = true
    const sent = await send()
    submit.disabled = false
    if (sent) {
      row.hidden = true
      show('success', 'You are on the list. Thank you!')
    } else {
      show('failure', 'That did not go through. Please try again.')
      form.querySelector('input[type="email"]').focus()
    }
  })
})
