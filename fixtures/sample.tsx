import React, { useState, useEffect, useCallback } from "react";

interface ButtonProps {
  label: string;
  onClick: () => void;
  disabled?: boolean;
  variant?: "primary" | "secondary" | "danger";
}

interface CardProps {
  title: string;
  children: React.ReactNode;
  footer?: React.ReactNode;
}

type Theme = "light" | "dark";

interface ThemeContextValue {
  theme: Theme;
  toggle: () => void;
}

const ThemeContext = React.createContext<ThemeContextValue>({
  theme: "light",
  toggle: () => {},
});

export function Button({ label, onClick, disabled = false, variant = "primary" }: ButtonProps) {
  return (
    <button
      className={`btn btn-${variant}`}
      onClick={onClick}
      disabled={disabled}
    >
      {label}
    </button>
  );
}

export function Card({ title, children, footer }: CardProps) {
  return (
    <div className="card">
      <div className="card-header">
        <h3>{title}</h3>
      </div>
      <div className="card-body">{children}</div>
      {footer && <div className="card-footer">{footer}</div>}
    </div>
  );
}

export function useTheme() {
  return React.useContext(ThemeContext);
}

export function useLocalStorage<T>(key: string, initial: T) {
  const [value, setValue] = useState<T>(() => {
    try {
      const item = window.localStorage.getItem(key);
      return item ? JSON.parse(item) : initial;
    } catch {
      return initial;
    }
  });

  const setStored = useCallback(
    (v: T) => {
      setValue(v);
      window.localStorage.setItem(key, JSON.stringify(v));
    },
    [key],
  );

  return [value, setStored] as const;
}

export class ErrorBoundary extends React.Component<
  { children: React.ReactNode },
  { hasError: boolean }
> {
  state = { hasError: false };

  static getDerivedStateFromError() {
    return { hasError: true };
  }

  render() {
    if (this.state.hasError) {
      return <div className="error-boundary">Something went wrong.</div>;
    }
    return this.props.children;
  }
}

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [theme, setTheme] = useLocalStorage<Theme>("theme", "light");

  const toggle = useCallback(() => {
    setTheme((t) => (t === "light" ? "dark" : "light"));
  }, [setTheme]);

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
  }, [theme]);

  return (
    <ThemeContext.Provider value={{ theme, toggle }}>
      {children}
    </ThemeContext.Provider>
  );
}

export const Avatar = ({ name, size = 40 }: { name: string; size?: number }) => (
  <div
    className="avatar"
    style={{ width: size, height: size }}
    aria-label={name}
  >
    {name[0].toUpperCase()}
  </div>
);
