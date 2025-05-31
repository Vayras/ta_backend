### Backend Setup

1.  **Clone the backend repository:**
    ```bash
    git clone [https://github.com/Vayras/ta_backend.git](https://github.com/Vayras/ta_backend.git)
    cd ta_backend
    ```

2.  **Run database migrations:**
    This command will set up the necessary database schema using SQLite.
    ```bash
    cargo run --bin migrate
    ```

3.  **Run the backend server:**
    This command will start the backend application with informational logging.
    ```bash
    RUST_LOG=info cargo run
    ```
    The backend server will typically be running on a port like `localhost:8000` (or as specified in your backend configuration).

---

### Frontend Setup


1.  **Clone the frontend repository:**
    Open a new terminal window or navigate to a different directory for the frontend.
    ```bash
    git https://github.com/Vayras/TAFE.git
    cd TAFE
    ```

2.  **Install frontend dependencies:**
    ```bash
    npm install
    ```

3.  **Run the frontend development server:**
    ```bash
    npm run dev
    ```
    This will usually start a development server, often on a port like `localhost:3000` or `localhost:5173` (Vite default). Check your terminal output for the exact address.
