document.querySelectorAll('.auto-submit')
  .forEach((input) => {
    input.addEventListener('change', () => {
      if (input.form) {
        input.form.submit()
      }
    })
  })
